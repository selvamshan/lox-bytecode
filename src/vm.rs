use crate::value;
use crate::value::*;
use crate::chunks::*;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}


pub struct VM {
    //chunk: Option<Chunk>,
    ip:usize,
    stack:Vec<Value>
}

impl  VM {
    pub fn new() -> Self{
        Self { 
            ip:0,
            stack: Vec::new(),
        }
    }

    // pub fn rest_stack(&mut self) {
    //     self.stack = Vec::new();
    // }

    pub fn free(&mut self) { }

    pub fn interpret(&mut self, chunk:&Chunk) -> InterpretResult {
        self.ip = 0;
        self.run(chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        loop {
            #[cfg(feature="debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {slot} ] ");

                }
                println!();

                chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte(&chunk);
            match instruction {             
                OpCode::OpReturn => {
                    println!("{}", self.stack.pop().unwrap());
                    return InterpretResult::Ok;
                }
                OpCode::OpConstant => {
                    let constant = self.read_constant(&chunk);
                    self.stack.push(constant);                   
                }, 
                OpCode::OpNegate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value);
                }  
                OpCode::OpAdd => self.binary_op(|a, b| a + b),               
                OpCode::OpSubtract => self.binary_op(|a, b| a - b), 
                OpCode::OpMultiply => self.binary_op(|a, b| a * b),  
                OpCode::OpDivide => self.binary_op(|a, b| a / b),         
            }
        }
    }

    fn read_byte(&mut self, chunk: &Chunk) -> OpCode {
        let val: OpCode = chunk.read(self.ip).into();
        self.ip += 1;
        val
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        let index = chunk.read(self.ip) as usize;
        self.ip += 1;
        chunk.get_constant(index)
    }

    fn binary_op<F>(&mut self,  f: F)
    where F: Fn(Value, Value) -> Value,   
    {      
         let b = self.stack.pop().unwrap();
         let a = self.stack.pop().unwrap();
         self.stack.push(f(a, b));
    }
    
  
    
}

