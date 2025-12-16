use std::cell::RefCell;
use std::rc::Rc;

use crate::chunks::*;
use crate::error::*;
use crate::function::*;
use crate::scanner::*;
use crate::token::*;
use crate::token_type::*;
use crate::value::*;

pub struct Compiler {
    rules: Vec<ParseRule>,
    parser: Parser,
    scanner: Scanner,
    result: RefCell<Rc<CompilerResult>>,
    current_class: RefCell<Option<Rc<ClassCompiler>>>
}

#[derive(PartialEq, Default)]
enum ChunkType {
    #[default]
    Script,
    Function,
    Method,
    Initializer,
}

#[derive(Default, PartialEq)]
struct UpvlaueData {
    is_local: bool,
    index: u8,
}

#[derive(Default)]
struct ClassCompiler {
    enclosing: RefCell<Option<Rc<ClassCompiler>>>,
    has_superclass: RefCell<bool>
}

impl ClassCompiler {
    fn new() -> Self {
        Self { 
            enclosing: RefCell::new(None),
            has_superclass: RefCell::new(false)
         }
    }
}

#[derive(Default)]
struct CompilerResult {
    chunk: RefCell<Chunk>,
    locals: RefCell<Vec<Local>>,
    scope_depth: RefCell<usize>,
    arity: RefCell<usize>,
    current_function: RefCell<String>,
    ctype: ChunkType,
    enclosing: RefCell<Option<Rc<CompilerResult>>>,
    upvalues: RefCell<Vec<UpvlaueData>>,   
}

enum FindResult {
    Uninitialized,
    NotFound,
    Depth(u8),
    ToManyvariables,
}

impl CompilerResult {
    fn new<T: Into<String>>(name: T, ctype: ChunkType) -> Self {
        let locals = RefCell::new(Vec::new());
        locals.borrow_mut().push( 
            if ctype != ChunkType::Function {
                  Local { 
                    name: Token { ttype: TokenType::This, lexeme: String::from("this"), line: 0 }, 
                    depth: Some(0), 
                    is_captured: false
               }
            } else {              
               Local {
                    name: Token::default(),
                    depth: Some(0),
                    is_captured: false,
                }
          }
        );
        Self {
            locals,
            current_function: RefCell::new(name.into()),
            ctype,
            ..Default::default()
        }
    }
    fn arity(&self) -> usize {
        *self.arity.borrow()
    }

    fn inc_arity(&self) -> usize {
        *self.arity.borrow_mut() += 1;
        *self.arity.borrow()
    }

    fn locals(&self) -> usize {
        self.locals.borrow().len()
    }

    fn find_variable(&self, name: &str) -> FindResult {
        for (e, v) in self.locals.borrow().iter().rev().enumerate() {
            if v.name.lexeme == *name {
                if v.depth.is_none() {
                    return FindResult::Uninitialized;
                }
                return FindResult::Depth((self.locals.borrow().len() - e - 1) as u8);
            }
        }
        FindResult::NotFound
    }

    fn capture(&self, index: usize) {
        let mut new_local = self.locals.borrow()[index].clone();
        new_local.is_captured = true;
        self.locals.borrow_mut()[index] = new_local;
    }

    fn resolve_local(&self, name: &Token) -> Result<Option<u8>, FindResult> {
        let find_result = self.find_variable(&name.lexeme);
        match find_result {
            FindResult::Uninitialized | FindResult::ToManyvariables => Err(find_result),
            FindResult::NotFound => Ok(None),
            FindResult::Depth(d) => Ok(Some(d)),
        }
    }

    fn add_upvalue(&self, index: u8, is_local: bool) -> Result<u8, FindResult> {
        let upvlaue = UpvlaueData { index, is_local };
        if let Some(pos) = self.upvalues.borrow().iter().position(|x| x == &upvlaue) {
            return Ok(pos as u8);
        }
        let upvalue_count = self.upvalues.borrow().len() as u8;
        if upvalue_count == 255 {
            return Err(FindResult::ToManyvariables);
        }
        self.upvalues.borrow_mut().push(upvlaue);
        Ok(upvalue_count)
    }

    fn resolve_upvalue(&self, name: &Token) -> Result<Option<u8>, FindResult> {
        if self.enclosing.borrow().is_none() {
            return Ok(None);
        }

        if let Some(depth) = self
            .enclosing
            .borrow()
            .as_ref()
            .unwrap()
            .resolve_local(name)?
        {
            self.enclosing
                .borrow()
                .as_ref()
                .unwrap()
                .capture(depth as usize);
            return Ok(Some(self.add_upvalue(depth, true)?));
        }

        match self
            .enclosing
            .borrow()
            .as_ref()
            .unwrap()
            .resolve_upvalue(name)?
        {
            None => Ok(None),
            Some(depth) => Ok(Some(self.add_upvalue(depth, false)?)),
        }
    }

    fn in_scope(&self) -> bool {
        *self.scope_depth.borrow() != 0
    }

    fn set_local_scope(&self) {
        let last = self.locals() - 1;
        let mut locals = self.locals.borrow_mut();
        locals[last].depth = Some(*self.scope_depth.borrow())
    }

    fn is_scope_popapable(&self) -> bool {
        !self.locals.borrow().is_empty()
            && self.locals.borrow().last().unwrap().depth.unwrap() > *self.scope_depth.borrow()
    }

    fn is_captured(&self) -> bool {
        self.locals.borrow().last().unwrap().is_captured
    }

    fn inc_scope(&self) {
        *self.scope_depth.borrow_mut() += 1;
    }

    fn dec_scope(&self) {
        *self.scope_depth.borrow_mut() -= 1;
    }

    fn pop(&self) {
        self.locals.borrow_mut().pop();
    }

    fn push(&self, local: Local) {
        self.locals.borrow_mut().push(local)
    }

    fn write(&self, byte: u8, line: usize) {
        self.chunk.borrow_mut().write(byte, line);
    }

    fn count(&self) -> usize {
        self.chunk.borrow().count()
    }

    fn add_constant(&self, value: Value) -> Option<u8> {
        self.chunk.borrow_mut().add_constant(value)
    }

    fn write_at(&self, offset: usize, byte: u8) {
        self.chunk.borrow_mut().write_at(offset, byte);
    }

    #[cfg(feature = "debug_print_code")]
    fn disassemble<T: Into<String>>(&self, name: T) {
        self.chunk.borrow().disassemble(name);
    }
}

#[derive(Default)]
pub struct Parser {
    current: Token,
    previous: Token,
    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}

#[derive(Clone, Copy)]
struct ParseRule {
    prefix: Option<fn(&mut Compiler, bool)>,
    infix: Option<fn(&mut Compiler, bool)>,
    precedence: Precedence,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   //  == !=
    Comparison, // < > <= =>
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

#[derive(Clone)]
struct Local {
    name: Token,
    depth: Option<usize>,
    is_captured: bool,
}

impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
        match value {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => panic!("Cannot covert {value} into precedence"),
        }
    }
}

impl Precedence {
    fn next(self) -> Self {
        if self == Precedence::Primary {
            panic!("no next precedence after Primary")
        }
        let p = self as usize;
        (p + 1).into()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut rules = vec![
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            };
            TokenType::NumberOfTokes as usize
        ];

        rules[TokenType::LeftParen as usize] = ParseRule {
            prefix: Some(Compiler::grouping),
            infix: Some(Compiler::call),
            precedence: Precedence::Call,
        };
        rules[TokenType::Minus as usize] = ParseRule {
            prefix: Some(|c, b| c.unary(b)),
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Term,
        };
        rules[TokenType::Plus as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Term,
        };
        rules[TokenType::Slash as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Factor,
        };
        rules[TokenType::Star as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Factor,
        };
        rules[TokenType::Number as usize].prefix = Some(|c, b| c.number(b));
        rules[TokenType::Nil as usize].prefix = Some(|c, b| c.literal(b));
        rules[TokenType::True as usize].prefix = Some(|c, b| c.literal(b));
        rules[TokenType::False as usize].prefix = Some(|c, b| c.literal(b));
        rules[TokenType::Bang as usize].prefix = Some(|c, b| c.unary(b));

        rules[TokenType::BangEqual as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Equality,
        };
        rules[TokenType::Equal as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Equality,
        };

        rules[TokenType::Greater as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Comparison,
        };
        rules[TokenType::GreaterEqual as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Comparison,
        };

        rules[TokenType::Less as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Comparison,
        };
        rules[TokenType::LessEqual as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.binary(b)),
            precedence: Precedence::Comparison,
        };
        rules[TokenType::String as usize].prefix = Some(|c, b| c.string(b));
        rules[TokenType::Identifier as usize].prefix = Some(|c, b| c.variable(b));
        rules[TokenType::And as usize] = ParseRule {
            prefix: None,
            infix: Some(|c, b| c.and(b)),
            precedence: Precedence::And,
        };
        rules[TokenType::Or as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::or),
            precedence: Precedence::Or,
        };
        rules[TokenType::Dot as usize] = ParseRule {
            prefix: None,
            infix: Some(Compiler::dot),
            precedence: Precedence::Call,
        };
        rules[TokenType::This as usize].prefix = Some(Compiler::this);
        rules[TokenType::Super as usize].prefix = Some(Compiler::super_);

        Self {
            rules,
            parser: Parser::default(),
            scanner: Scanner::new(""),
            result: RefCell::new(Rc::new(CompilerResult::default())),
            current_class: RefCell::new(None)
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<Function, InterpretResult> {
        self.result.borrow().push(Local {
            name: Token::default(),
            depth: Some(0),
            is_captured: false,
        });
        self.scanner = Scanner::new(source);
        self.advance();

        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }

        self.end_compiler();

        if *self.parser.had_error.borrow() {
            Err(InterpretResult::CompileError)
        } else {
            let result = self.result.replace(Rc::new(CompilerResult::default()));
            let chunk = result.chunk.replace(Chunk::new());
            Ok(Function::toplevel(&Rc::new(chunk)))
        }
    }

    pub fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.ttype != TokenType::Error {
                break;
            }
            let message = self.parser.current.lexeme.as_str();
            self.error_at_current(message);
        }
    }

    fn consume(&mut self, ttype: TokenType, message: &str) {
        if self.parser.current.ttype == ttype {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn check(&self, ttype: TokenType) -> bool {
        self.parser.current.ttype == ttype
    }

    fn is_match(&mut self, ttype: TokenType) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn emit_byte<T: Into<u8>>(&mut self, byte: T) {
        self.result
            .borrow()
            .write(byte.into(), self.parser.previous.line);
    }

    fn emit_bytes<T: Into<u8>, U: Into<u8>>(&mut self, byte1: T, byte2: U) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);

        let offset = self.result.borrow().count() + 2 - loop_start;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.result.borrow().count() - 2
    }

    fn emit_return(&mut self) {
        if self.result.borrow().ctype == ChunkType::Initializer {
            self.emit_bytes(OpCode::GetLocal, 0);
        } else {
             self.emit_byte(OpCode::Nil);
        }       
        self.emit_byte(OpCode::Return);
    }

    fn make_costant(&mut self, value: Value) -> u8 {
        if let Some(constant) = self.result.borrow().add_constant(value) {
            constant
        } else {
            self.error("Too many constants in one chunk");
            0
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_costant(value);
        self.emit_bytes(OpCode::Constant, constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.result.borrow().count() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too mutch code to jump over.");
        }

        self.result
            .borrow()
            .write_at(offset, ((jump >> 8) & 0xff) as u8);
        self.result
            .borrow()
            .write_at(offset + 1, (jump & 0xff) as u8);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "debug_print_code")]
        {
            let name = if self.result.borrow().current_function.borrow().is_empty() {
                "<script>".to_string()
            } else {
                self.result.borrow().current_function.borrow().clone()
            };
            if !*self.parser.had_error.borrow() {
                self.result.borrow().disassemble(name);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.result.borrow().inc_scope();
    }

    fn end_scope(&mut self) {
        self.result.borrow().dec_scope();

        while self.result.borrow().is_scope_popapable() {
            if self.result.borrow().is_captured() {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.result.borrow().pop();
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.ttype;
        //let rule = self.get_rule(operator_type);
        let rule = &self.rules[operator_type as usize];

        self.parse_precedence(rule.precedence.next());

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::Equal => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),

            _ => todo!(),
        }
    }

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call, arg_count);
    }

    fn dot(&mut self, can_assign:bool) {
        self.consume(TokenType::Identifier, "Expect property name after '.'");
        let name = self.identifier_constant(&self.parser.previous.clone());

        if can_assign && self.is_match(TokenType::Assign) {
            self.expression();
            self.emit_bytes(OpCode::SetProperty, name);            
        } else if self.is_match(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::Invoke, name);
            self.emit_byte(arg_count);
        }
        else {
            self.emit_bytes(OpCode::GetProperty, name);
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.ttype;
        match operator_type {
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::False => self.emit_byte(OpCode::False),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expresssion");
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.parser.previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value))
    }

    fn or(&mut self, _: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn string(&mut self, _can_assign: bool) {
        let len = self.parser.previous.lexeme.len() - 1;
        let string = self.parser.previous.lexeme[1..len].to_string();
        self.emit_constant(Value::Str(string));
    }

    fn resolve_local(&self, name: &Token) -> Option<u8> {
        match self.result.borrow().resolve_local(name) {
            Err(FindResult::Uninitialized) => {
                self.error("Cannot read local variable in its own initializer.");
                None
            }
            Ok(val) => val,
            _ => panic!("invalid return from resolve local"),
        }
    }

    fn resolve_upvalue(&self, name: &Token) -> Option<u8> {
        match self.result.borrow().resolve_upvalue(name) {
            Err(FindResult::ToManyvariables) => {
                self.error("TODO - error message");
                None
            }
            Ok(val) => val,
            _ => panic!("Invalid return from resolve_upvalue"),
        }
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let (arg, get_op, set_op) = if let Some(local_arg) = self.resolve_local(name) {
            (local_arg, OpCode::GetLocal, OpCode::SetLocal)
        } else if let Some(upvalue_arg) = self.resolve_upvalue(name) {
            (upvalue_arg, OpCode::GetUpvalue, OpCode::SetUpvalue)
        } else {
            (
                self.identifier_constant(name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            )
        };

        if can_assign && self.is_match(TokenType::Assign) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let name = &self.parser.previous.clone();
        self.named_variable(name, can_assign);
    }

    fn this(&mut self, _can_assign: bool) {
        if self.current_class.borrow().is_none() {
            self.error("Can't use 'this' outside of a class");
        }
        
        self.variable(false);
    }

    fn super_(&mut self, _can_assign:bool) {
        match self.current_class.borrow().as_ref() {
            None => self.error("Can't use 'super' outside of a class"),
            Some(cc) => {
                if !*cc.has_superclass.borrow() {
                    self.error("Can't use 'super' in a class with no superclass.");
                }
            }
        }
        
        self.consume(TokenType::Dot, "Expect '.' after 'super'.");
        self.consume(TokenType::Identifier, "Expect superclass method name.");
        let name = self.identifier_constant(&self.parser.previous.clone());
        //&Token::new("super")
        self.named_variable(&Token::new("this"), false);
        if self.is_match(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.named_variable(&Token::new("super"), false);
            self.emit_bytes(OpCode::SuperInoke, name);
            self.emit_byte(arg_count);
        } else {
            self.named_variable(&Token::new("super"), false);
            self.emit_bytes(OpCode::GetSuper, name);
        }
        
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.ttype;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => unimplemented!("nope"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self.rules[self.parser.previous.ttype as usize].prefix {
            let can_assign = precedence <= Precedence::Assignment;
            prefix_rule(self, can_assign);
            while precedence <= self.rules[self.parser.current.ttype as usize].precedence {
                self.advance();
                if let Some(infix_rule) = self.rules[self.parser.previous.ttype as usize].infix {
                    infix_rule(self, can_assign);
                }
                if can_assign && self.is_match(TokenType::Assign) {
                    self.error("Invalid assigment target");
                }
            }
        } else {
            self.error("Expect Expression.");
        }
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        self.make_costant(Value::Str(name.lexeme.clone()))
    }

    fn add_local(&mut self, name: &Token) {
        if self.result.borrow().locals() >= u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }
        self.result.borrow().push(Local {
            name: name.clone(),
            depth: None,
            is_captured: false,
        });
    }

    fn declar_variable(&mut self) {
        if self.result.borrow().in_scope() {
            let name = &self.parser.previous.lexeme;
            if let FindResult::Depth(_) = self.result.borrow().find_variable(name) {
                self.error("Already a variable with this name in this scope.");
            } else {
                self.add_local(&self.parser.previous.clone())
            }
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declar_variable();
        if !self.result.borrow().in_scope() {
            self.identifier_constant(&self.parser.previous.clone())
        } else {
            0
        }
    }

    fn mark_initialized(&mut self) {
        if self.result.borrow().in_scope() {
            self.result.borrow().set_local_scope();
        }
    }

    fn define_variable(&mut self, global: u8) {
        if !self.result.borrow().in_scope() {
            self.emit_bytes(OpCode::DefineGlobal, global);
        } else {
            self.mark_initialized();
        }
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.is_match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);
        if self.is_match(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Excpect '}' after block");
    }

    fn function(&mut self, ctype:ChunkType) {
        let prev_complier = self.result.replace(Rc::new(CompilerResult::new(
            self.parser.previous.lexeme.clone(),
            ctype
        )));

        self.result.borrow().enclosing.replace(Some(prev_complier));

        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after function name");
        if !self.check(TokenType::RightParen) {
            loop {
                if self.result.borrow().inc_arity() > 255 {
                    self.error("Can't have more than 255 parameters.");
                }
                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);
                if !self.is_match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body");

        self.block();

        self.end_compiler();
        let arity = self.result.borrow().arity();
        let prev_complier = self.result.borrow().enclosing.replace(None).unwrap();
        let result = self.result.replace(prev_complier);

        if !*self.parser.had_error.borrow() {
            let chunk = result.chunk.replace(Chunk::new());
            let func = Function::new(
                &*result.current_function.borrow(),
                arity,
                &Rc::new(chunk),
                result.upvalues.borrow().len(),
            );

            let constant = self.make_costant(Value::Func(Rc::new(func)));
            self.emit_bytes(OpCode::Closure, constant);

            for upvalue in result.upvalues.borrow().iter() {
                self.emit_byte(if upvalue.is_local { 1 } else { 0 });
                self.emit_byte(upvalue.index);
            }
        }
    }

    fn method(&mut self) {
        self.consume(TokenType::Identifier, "Expect class name.");
        let parse_token = self.parser.previous.clone();
        let constant = self.identifier_constant(&parse_token);

        self.function(if parse_token.lexeme == "init" {
        ChunkType::Initializer
        } else {
            ChunkType::Method
        });
        self.emit_bytes(OpCode::Method, constant);
    }

    fn class_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect class name.");
        let class_name =  self.parser.previous.clone();
        let name_constant = self.identifier_constant(&class_name);
        self.declar_variable();

        self.emit_bytes(OpCode::Class, name_constant);
        self.define_variable(name_constant);
        
        let prev = self        
        .current_class
        .replace(Some(Rc::new(ClassCompiler::new())));

        self
        .current_class
        .borrow()
        .as_ref()
        .unwrap()
        .enclosing        
        .replace(prev);

        if self.is_match(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect supreclass name.");
            self.variable(false);
            let prev = self.parser.previous.clone();
            if class_name.lexeme == prev.lexeme {
                self.error("A class can't inherit from itself");
            }
            self.begin_scope();
            self.add_local(&Token::new("super"));
            self.define_variable(0);
            self.named_variable(&class_name, false);
            self.emit_byte(OpCode::Inherit);
            self.current_class.borrow().as_ref().unwrap().has_superclass.replace(true);
        }

        self.named_variable(&class_name, false);
        self.consume(TokenType::LeftBrace, "Expected '{' before class body");
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof){
            self.method();
        }
        self.consume(TokenType::RightBrace, "Expected '}' before class body");
        self.emit_byte(OpCode::Pop);
        if *self.current_class.borrow().as_ref().unwrap().has_superclass.borrow() {
            self.end_scope();
        }

        let prev =  self
        .current_class
        .borrow()
        .as_ref()
        .unwrap()
        .enclosing        
        .replace(None);
        self.current_class.replace(prev);
        
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(ChunkType::Function);
        self.define_variable(global);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.is_match(TokenType::Assign) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.is_match(TokenType::SemiColon) {
            // No initializer.
        } else if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.result.borrow().count();
        let exit_jump = if !self.is_match(TokenType::SemiColon) {
            self.expression();
            self.consume(TokenType::SemiColon, "Expect ';' after loop condition.");

            let result = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::Pop);
            Some(result)
        } else {
            None
        };

        if !self.is_match(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.result.borrow().count();
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);
        if let Some(exit) = exit_jump {
            self.patch_jump(exit);
            self.emit_byte(OpCode::Pop);
        }
        self.end_scope();
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn return_statement(&mut self) {
        if self.result.borrow().ctype == ChunkType::Script {
            self.error("Can't return from top-level code.");
        }
        if self.is_match(TokenType::SemiColon) {
            self.emit_return();
        } else {
            if self.result.borrow().ctype == ChunkType::Initializer {
                self.error("Can't return a value form a initializer.");
            }
            self.expression();
            self.consume(TokenType::SemiColon, "Expect ';' after return value");
            self.emit_byte(OpCode::Return);
        }
    }

    fn while_statment(&mut self) {
        let loop_start = self.result.borrow().count();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Excepect ')' after conditions.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode.replace(false);

        while self.parser.current.ttype != TokenType::Eof {
            if self.parser.previous.ttype == TokenType::SemiColon {
                return;
            }

            match self.parser.current.ttype {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.is_match(TokenType::Class) {
            self.class_declaration();
        } else if self.is_match(TokenType::Fun) {
            self.fun_declaration();
        } else if self.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if *self.parser.panic_mode.borrow() {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.is_match(TokenType::Print) {
            self.print_statement();
        } else if self.is_match(TokenType::For) {
            self.for_statement()
        } else if self.is_match(TokenType::If) {
            self.if_statement();
        } else if self.is_match(TokenType::Return) {
            self.return_statement();
        } else if self.is_match(TokenType::While) {
            self.while_statment();
        } else if self.is_match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn error_at_current(&self, message: &str) {
        self.error_at(&self.parser.current, message)
    }

    fn error(&self, message: &str) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&self, token: &Token, message: &str) {
        if *self.parser.panic_mode.borrow() {
            return;
        }
        self.parser.panic_mode.replace(true);
        eprint!("[line {:}] Error", token.line);

        if token.ttype == TokenType::Eof {
            eprint!(" at end");
        } else if token.ttype == TokenType::Error {
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {}", message);
        self.parser.had_error.replace(true);
    }
}
