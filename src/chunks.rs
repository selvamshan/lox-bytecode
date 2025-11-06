use std::fmt::{Display, Result};

use crate::value::*;


pub enum OpCode {
    OpConstant = 0,
    OpReturn = 1,
}

// impl Display for OpCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
//         match {
//             OpCode::OpReturn == 0 => write!(f, "OP_RETURN"),
//         };
//     }
// }

pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new(),
            constants: ValueArray::new()
         }
    }

    pub fn write(&mut self, byte:u8) {
        self.code.push(byte)
    }

    pub fn write_opcode(&mut self, code:OpCode) {
        self.code.push(code.into());
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
        self.constants.free();
        self.constants = ValueArray::new();
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value) as u8     
    }

    pub fn disassemble<T:ToString>(&self, name: T)
    where T: Display {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    fn disassemble_instruction(&self, offset:usize) -> usize {
        print!("{:04} ", offset);
        let instruction:OpCode = self.code[offset].into();
        match instruction {
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset),  
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),           
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
            _ => unimplemented!("Invalid opcode")
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}

