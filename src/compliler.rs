
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
    prefix: Option<fn(&mut Compiler)>,
    infix : Option<fn(&mut Compiler)>,
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

    // fn previous(self) -> Self {
    //     if self == Precedence::None {
    //         panic!("no previous precedence berfore None")
    //     } 
    //     let p = self as usize;
    //     (p - 1).into()       
        
    // }
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
                prefix:Some(|c| c.grouping()), 
                infix: None, 
                precedence: Precedence::None };
            rules[TokenType::Minus as usize] = ParseRule { 
                prefix:Some(|c| c.unary()),
                infix: Some(|c| c.binary()), 
                precedence: Precedence::Term };
            rules[TokenType::Plus as usize] = ParseRule { 
                prefix:None, 
                infix: Some(|c| c.binary()), 
                precedence: Precedence::Term };            
            rules[TokenType::Slash as usize]= ParseRule {
                 prefix:None, infix: Some(|c| c.binary()), precedence: Precedence::Factor };
            rules[TokenType::Star as usize] = ParseRule { 
                prefix:None, infix: Some(|c| c.binary()), precedence: Precedence::Factor };
            rules[TokenType::Number as usize].prefix = Some(|c| c.number()); 
            rules[TokenType::Nil as usize].prefix = Some(|c|c.literal());
            rules[TokenType::True as usize].prefix = Some(|c|c.literal());
            rules[TokenType::False as usize].prefix = Some(|c|c.literal());
          
        Self { 
            parser: Parser::default(),
            scanner: Scanner::new(&"".to_string()),
            chunk,
            rules
          }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult>{
        self.scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");
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

    fn emit_byte(&mut self, byte:u8) {
        self.chunk.write(byte, self.parser.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
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

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature="debug_print_code")]
        if !*self.parser.had_error.borrow() {
            self.chunk.disassemble("code");
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.ttype;
        //let rule = self.get_rule(operator_type);
        let rule = &self.rules[operator_type as usize];       
       
        self.parse_precedence(rule.precedence.next());

        match operator_type {
           TokenType::Plus => self.emit_byte(OpCode::Add.into()), 
           TokenType::Minus => self.emit_byte(OpCode::Subtract.into()), 
           TokenType::Star => self.emit_byte(OpCode::Multiply.into()), 
           TokenType::Slash => self.emit_byte(OpCode::Divide.into()), 
           _ => todo!()
        }
    }

    fn literal(&mut self) {
        let operator_type = self.parser.previous.ttype;
        match operator_type {
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::False => self.emit_byte(OpCode::False.into()),
            _ => unreachable!()
        }
    }

    fn grouping(&mut self)  {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expresssion");
    }

    fn number(&mut self) {
       let value = self.parser.previous.lexeme.parse::<f64>().unwrap();
       self.emit_constant(Value::Number(value))
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.ttype;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            _ => unimplemented!("nope")
        }
        
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self
            .rules[self.parser.previous.ttype as usize].prefix {
               prefix_rule(self);
               while precedence <= self.rules[self.parser.current.ttype as usize].precedence {
                 self.advance();
                 if let Some(infix_rule) = self.rules[self.parser.previous.ttype as usize].infix {
                    infix_rule(self)
                 }
               }
               return;
        } else {
            self.error("Expect Expression.");
        }
    }

    
 
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
       
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
        eprint!("[line {:4} Error", token.line);

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