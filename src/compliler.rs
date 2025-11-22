
use std::cell:: RefCell;

use crate::{scanner::*, vm::InterpretResult};
use crate::chunks::*;
use crate::token::*;
use crate::token_type::*;
use crate::value::*;


pub struct Compiler<'a>{
    parser: Parser,
    scanner: Scanner,
    chunk: &'a mut Chunk,
    rules: Vec<ParseRule>,
    locals: RefCell<Vec<Local>>,
    scope_depth: usize,
}

#[derive(Default)]
pub struct Parser {
    current: Token,
    previous: Token,
    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>
}

#[derive(Clone, Copy)]
struct ParseRule {
    prefix: Option<fn(&mut Compiler, bool)>,
    infix : Option<fn(&mut Compiler, bool)>,
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
struct Local{
    name: Token,
    depth: Option<usize>
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
            _ => panic!("Cannot covert {value} into precedence")
        }
    }
}

impl Precedence {
    fn next(self) ->  Self {
        if self == Precedence::Primary {
            panic!("no next precedence after Primary")
        } 
        let p = self as usize;
        (p + 1).into()
        
    }
    
    /*
    fn previous(self) -> Self {
        if self == Precedence::None {
            panic!("no previous precedence berfore None")
        } 
        let p = self as usize;
        (p - 1).into()       
        
    }
    */
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        let mut rules = vec![
            ParseRule {
              prefix:None, 
              infix: None,
              precedence: Precedence::None, 
            }; 
            TokenType::NumberOfTokes as usize];

            rules[TokenType::LeftParen as usize] = ParseRule { 
                prefix:Some(|c, b| c.grouping(b)), 
                infix: None, 
                precedence: Precedence::None };
            rules[TokenType::Minus as usize] = ParseRule { 
                prefix:Some(|c, b| c.unary(b)),
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Term };
            rules[TokenType::Plus as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Term };            
            rules[TokenType::Slash as usize]= ParseRule {
                 prefix:None,
                  infix: Some(|c, b| c.binary(b)), 
                  precedence: Precedence::Factor };
            rules[TokenType::Star as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Factor };
            rules[TokenType::Number as usize].prefix = Some(|c, b| c.number(b)); 
            rules[TokenType::Nil as usize].prefix = Some(|c,b|c.literal(b));
            rules[TokenType::True as usize].prefix = Some(|c, b|c.literal(b));
            rules[TokenType::False as usize].prefix = Some(|c, b|c.literal(b));
            rules[TokenType::Bang as usize].prefix = Some(|c, b|c.unary(b));

            rules[TokenType::BangEqual as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Equality };          
            rules[TokenType::Equal as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Equality };

            rules[TokenType::Greater as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Comparison };
            rules[TokenType::GreaterEqual as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Comparison };

            rules[TokenType::Less as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Comparison };
            rules[TokenType::LessEqual as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c, b| c.binary(b)), 
                precedence: Precedence::Comparison };
            rules[TokenType::String as usize].prefix = Some(|c, b| c.string(b)); 
            rules[TokenType::Identifier as usize].prefix = Some(|c, b| c.vairable(b));
          
        Self { 
            parser: Parser::default(),
            scanner: Scanner::new(&"".to_string()),
            chunk,
            rules,
            locals: RefCell::new(Vec::new()),
            scope_depth: 0,
          }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult>{
        self.scanner = Scanner::new(source);
        self.advance();
        
        while !self.is_match(TokenType::Eof) {
            self.declaration();
        }
       
        self.end_compiler();

        if *self.parser.had_error.borrow() {
            Err(InterpretResult::CompileError)
        } else {
            Ok(())
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
        self.error_at_current( message);
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

    fn emit_byte(&mut self, byte:u8) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
    }

    fn emit_jump(&mut self, instruction:OpCode) -> usize {
        self.emit_byte(instruction.into());
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.chunk.count() -2
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }

    fn make_costant(&mut self, value:Value) -> u8 {
       match self.chunk.add_constant(value){
         Some(constant) => constant,
         None => {
            self.error(&"Too many constants in one chunk");
            0
         }
       }
    }

    fn emit_constant(&mut self, value:Value) {
        let constant = self.make_costant(value);
        self.emit_bytes(OpCode::Constant, constant);
    }

    fn patch_jump(&mut self, offset:usize) {
        let jump = self.chunk.count() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too mutch code to jump over.");
        }

        self.chunk.write_at(offset, ((jump >> 8) & 0xff) as u8);
        self.chunk.write_at(offset + 1, (jump & 0xff) as u8);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature="debug_print_code")]
        if !*self.parser.had_error.borrow() {
            self.chunk.disassemble("code");
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while self.locals.borrow().len() > 0 && 
        self.locals.borrow().last().unwrap().depth.unwrap() > self.scope_depth {
            self.emit_byte(OpCode::Pop.into());
            self.locals.borrow_mut().pop();
        }
    }

    fn binary(&mut self, _can_assign:bool) {
        let operator_type = self.parser.previous.ttype;
        //let rule = self.get_rule(operator_type);
        let rule = &self.rules[operator_type as usize];       
       
        self.parse_precedence(rule.precedence.next());

        match operator_type {
           TokenType::Plus => self.emit_byte(OpCode::Add.into()), 
           TokenType::Minus => self.emit_byte(OpCode::Subtract.into()), 
           TokenType::Star => self.emit_byte(OpCode::Multiply.into()), 
           TokenType::Slash => self.emit_byte(OpCode::Divide.into()), 
           TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not.into()), 
           TokenType::Equal => self.emit_byte(OpCode::Equal.into()), 
           TokenType::Greater => self.emit_byte(OpCode::Greater.into()), 
           TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not.into()), 
           TokenType::Less => self.emit_byte(OpCode::Less.into()), 
           TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not.into()), 
           
           _ => todo!()
        }
    }

    fn literal(&mut self, _can_assign:bool) {
        let operator_type = self.parser.previous.ttype;
        match operator_type {
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::False => self.emit_byte(OpCode::False.into()),
            _ => unreachable!()
        }
    }

    fn grouping(&mut self, _can_assign:bool)  {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expresssion");
    }

    fn number(&mut self, _can_assign:bool) {
       let value = self.parser.previous.lexeme.parse::<f64>().unwrap();
       self.emit_constant(Value::Number(value))
    }

    fn string(&mut self, _can_assign:bool) {
        let len = self.parser.previous.lexeme.len() - 1;
        let string = self.parser.previous.lexeme[1..len].to_string();
        self.emit_constant(Value::Str(string));
    }
    fn resolve_local(&mut self,  name:&Token) -> Option<u8> {
        let len = self.locals.borrow().len();
        for i in (0..len).rev() {
            // clone the local entry while the borrow is active, then drop the borrow
            let local = self.locals.borrow()[i].clone();
            if local.name.lexeme == name.lexeme {
                if local.depth.is_none() {
                    // no RefCell borrow is active here, so calling self.error is safe
                    self.error("Cannot read local variable in its own initializer.");
                }
                return Some((len - 1 - i) as u8);
            }
        }
        None
    }
    

    fn named_variable(&mut self, name:&Token, can_assign:bool) {
        
        let (arg, get_op, set_op) = if let Some(local_arg) = self.resolve_local(name) {
            (local_arg, OpCode::GetLocal, OpCode::SetLocal)
        } else {
            (self.identifier_constant(name), OpCode::GetGlobal, OpCode::SetGlobal)
        };       

        
        if can_assign && self.is_match(TokenType::Assign) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg);
        }
    }

    fn vairable(&mut self, can_assign:bool) {
        let name = &self.parser.previous.clone();
        self.named_variable(&name, can_assign);
    }

    fn unary(&mut self, _can_assign:bool) {
        let operator_type = self.parser.previous.ttype;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unimplemented!("nope")
        }
        
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self
            .rules[self.parser.previous.ttype as usize].prefix {
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
               return;
        } else {
            self.error("Expect Expression.");
        }
    }

   fn identifier_constant(&mut self, name:&Token) -> u8 {
        self.make_costant(Value::Str(name.lexeme.clone()))
    }
  

    fn add_local(&mut self, name:&Token) {
        if self.locals.borrow().len() >= u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }
        self.locals.borrow_mut().push(Local {
            name: name.clone(),
            depth: None
        });
    }

    fn declar_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }  
        // Take a snapshot of locals to avoid holding the RefCell borrow while calling self.error,
        // since self.error requires a mutable borrow of self.
        let locals_snapshot: Vec<(usize, String)> = self
            .locals
            .borrow()
            .iter()
            .rev()
            .map(|local| (local.depth.unwrap(), local.name.lexeme.clone()))
            .collect();

        for (depth, lexeme) in locals_snapshot {
            if depth != usize::MAX && depth < self.scope_depth {
                return;
            }
            if lexeme == self.parser.previous.lexeme {
                self.error("Already a variable with this name in this scope.");
            }
        }
        // let name = self.parser.previous.lexeme.clone();
        // if self.locals.borrow().iter().filter(|x| *x.name.lexeme == name).count() > 0 {
        //     self.error("Already a variable with this name in this scope.");
        // }
        self.add_local(&self.parser.previous.clone())

    }

    fn parse_variable(&mut self, error_message:&str) -> u8 {
        self.consume(TokenType::Identifier, error_message); 

        self.declar_variable();
        if self.scope_depth == 0 {
            self.identifier_constant(&self.parser.previous.clone())
        } else {
            0
        }        
    }    

    fn mark_initialized(&mut self) {
       
        let len = self.locals.borrow().len();
        self.locals.borrow_mut()[len - 1].depth = Some(self.scope_depth);
       
    }

    fn define_variable(&mut self, global:u8) {
        if self.scope_depth == 0 {
            self.emit_bytes(OpCode::DefineGlobal, global);
        } else {
            self. mark_initialized();
        }
        
    }
 
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);       
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let  then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.statement();
        
        self.patch_jump(then_jump);
    }


    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Excpect '}' after block");
    }

    fn var_declaration(&mut self)  {
        let global = self.parse_variable("Expect variable name.");
        if self.is_match(TokenType::Assign) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        self.consume(TokenType::SemiColon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn print_statement(&mut self) {      
        self.expression();       
        self.consume(TokenType::SemiColon, "Expect ';' after value.");        
        self.emit_byte(OpCode::Print.into());       
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode.replace(false);

        while self.parser.current.ttype != TokenType::Eof {
            if self.parser.previous.ttype == TokenType::SemiColon {
                return;
            }

            match self.parser.current.ttype {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn declaration(&mut self) { 
        if self.is_match(TokenType::Var) {
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
        } else if self.is_match(TokenType::If) {
            self.if_statement();
        } else if self.is_match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }
 
    fn error_at_current(&self, message:&str) {
        self.error_at(&self.parser.current, message)
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&self, token: &Token, message:&str) {
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