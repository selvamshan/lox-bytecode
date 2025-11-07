use std::fmt::{Display, Result};

use crate::value::*;


pub enum OpCode {
    OpConstant = 0,
    OpReturn,
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,    
}



pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: ValueArray
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new()
         }
    }

    pub fn write(&mut self, byte:u8, line:usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_opcode(&mut self, code:OpCode, line:usize) {
        self.code.push(code.into());
        self.lines.push(line);
    }

    pub fn read(&self, ip:usize) -> u8 {
        self.code[ip]
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
        self.constants.free();
        self.constants = ValueArray::new();
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value) as u8     
    }

    pub fn get_constant(&self, index:usize) -> Value {
        self.constants.read_value(index)
    }

    pub fn disassemble<T:ToString>(&self, name: T)
    where T: Display {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    pub fn disassemble_instruction(&self, offset:usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset-1] {
            print!("   | ")
        } else {
            print!("{:4} ", self.lines[offset]);
        }
        let instruction:OpCode = self.code[offset].into();
        match instruction {
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset),  
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset), 
            OpCode::OpNegate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::OpAdd => self.simple_instruction("OP_ADD", offset),
            OpCode::OpSubtract => self.simple_instruction("OP_SUBTRACT", offset),   
            OpCode::OpMultiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::OpDivide =>  self.simple_instruction("OP_DIVIDE", offset) ,   
        }
    }

    fn simple_instruction(&self, name:&str, offset:usize) -> usize {
        println!("{name}");
         offset + 1
    }

    fn constant_instruction(&self, name:&str, offset:usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{:-16} {:4} '", name, constant);
        self.constants.print_value(constant as usize);
        println!("'");
        return  offset + 2;

    }
 }

 impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{:?}", self.code)
    }
 }

impl From<u8> for OpCode {
    fn from(code: u8) -> Self {
        match code {
            0 => OpCode::OpConstant,
            1 => OpCode::OpReturn,
            2 => OpCode::OpNegate,
            3 => OpCode::OpAdd,
            4 => OpCode::OpSubtract,
            5 => OpCode::OpMultiply,
            6 => OpCode::OpDivide,
            _ => unimplemented!("Invalid opcode")
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}

