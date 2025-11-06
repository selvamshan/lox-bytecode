use std::fmt::{Display, Result};


pub enum OpCode {
    OpReturn = 0,
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
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn write_opcode(&mut self, byte:u8) {
        self.code.push(byte);
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
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
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),           
        }
    }

    fn simple_instruction(&self, name:&str, offset:usize) -> usize {
        println!("{name}");

         offset + 1
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
            0 => OpCode::OpReturn,
            _ => unimplemented!("Invalid opcode")
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}

