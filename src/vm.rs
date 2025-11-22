use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::value::*;
use crate::chunks::*;
use crate::compliler::*;

pub enum InterpretResult {
    _Ok,
    CompileError,
    RuntimeError,
}


pub struct VM {   
    ip:usize,
    stack:Vec<Value>,  
    chunk: Rc<Chunk>,  
    globals: HashMap<String, Value>,
}

impl  VM {
    pub fn new() -> Self{
        Self { 
            ip:0,
            stack: Vec::new() , 
            chunk: Rc::new(Chunk::new()),  
            globals: HashMap::new(),        
        }
    }

    pub fn reset_stack(&mut self) {
        self.stack.clear();
    }
    

    pub fn interpret(&mut self, source:&str) -> Result<(),InterpretResult> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(&mut chunk);
        compiler.compile(source)?;

        self.ip = 0; 
        self.chunk = Rc::new(chunk);
        self.run()     
    
    }

    fn run(&mut self) ->  Result<(), InterpretResult> {
        loop {
            #[cfg(feature="debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {:?} ] ", slot.to_string());

                }
                println!();

                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte().into();
            match instruction {   
                OpCode::Print => {                                      
                    println!("{}", self.pop());
                } 
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if self.peek(0).is_falsey() {
                        self.ip += offset as usize;
                    }
                }         
                OpCode::Return => {
                    //println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);                   
                }, 
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Boolean(true)),
                OpCode::False => self.stack.push(Value::Boolean(false)),
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Negate => {
                    //if let Value::Number(_) = self.peek(0)
                    if self.peek(0).is_number()  {
                        let value = self.pop();
                        self.stack.push(-value);
                    } else {
                        return self.runtime_error(&"Operand must be a number");
                        
                    }                    
                },  
                OpCode::DefineGlobal => {
                    let constaant = self.read_constant().clone();
                    if let Value::Str(name) = constaant {
                        let value = self.pop();
                        self.globals.insert(name.clone(), value.clone());                        
                    } else {
                        panic!("DefineGlobal: constant is not a string");
                    }
                },
                OpCode::GetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(name) = constant {
                        if let Some(value) = self.globals.get(&name) {
                            self.stack.push(value.clone());
                        } else {
                            return self.runtime_error(&format!("Undefined variable '{:}'", name));
                        }
                    } else {
                        panic!("GetGlobal: constant is not a string");
                    } 
                },
                OpCode::SetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(name) = constant {
                        let p = self.peek(0).clone();
                        if let Entry::Occupied(mut o) = self.globals.entry(name.clone()) {
                            *o.get_mut() = p;
                        } else {
                            return self.runtime_error(&format!("Undefined variable '{:}'", name));  
                        }

                    } else {
                        panic!("SetGlobal: constant is not string");
                    }
                },
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    self.stack.push(self.stack[slot as usize].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    let value = self.peek(0).clone();
                    self.stack[slot as usize] = value;
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Boolean(a == b))
                },
                OpCode::Greater => self.binary_op(|a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.binary_op(|a, b| Value::Boolean(a < b))?,
                OpCode::Add => self.binary_op( |a, b| a + b)?,               
                OpCode::Subtract => self.binary_op( |a, b| a - b)?, 
                OpCode::Multiply => self.binary_op( |a, b| a * b)?,  
                OpCode::Divide => self.binary_op( |a, b| a / b)?, 
                OpCode::Not => {
                    let value = self.pop();
                    self.stack.push(Value::Boolean(value.is_falsey()))
                }

            }
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance:usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
    }

    fn read_byte(&mut self,) -> u8 {
        let val:u8 = self.chunk.read(self.ip);
        self.ip += 1;
        val
    }

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        //((self.chunk.read(self.ip -2) as u16) << 8) | (self.chunk.read(self.ip -1) as u16)
        self.chunk.get_jump_offset(self.ip - 2) as u16
    }

    fn read_constant(&mut self) -> &Value {
        let index = self.chunk.read(self.ip) as usize;
        self.ip += 1;
        &self.chunk.get_constant(index)
    }

    fn binary_op<F>(&mut self, f: F) -> Result<(), InterpretResult>
    where F: Fn(Value, Value) -> Value,   
    {    
        if self.peek(0).is_string() && self.peek(1).is_string() {
            let b = self.pop();
            let a = self.pop();        
            self.stack.push(f(a, b));
             Ok(())
        }
        else if self.peek(0).is_number()  && self.peek(1).is_number() {
            let b = self.pop();
            let a = self.pop();        
            self.stack.push(f(a, b));
         Ok(())
           
        } else {
            return self.runtime_error(&"Operands must be two numbers or two strings.")
        }
              
        
    }
    
    fn runtime_error<T:ToString>(&mut self, err_msg:&T) -> Result<(), InterpretResult> {
        let line = self.chunk.get_line(self.ip -1);
        eprintln!("{}", err_msg.to_string());
        eprintln!("[Line {:}] in script", line);
        self.reset_stack();
        Err(InterpretResult::RuntimeError)
    }
  
    
}

