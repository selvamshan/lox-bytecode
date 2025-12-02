use std::fmt::Display;
use std::rc::Rc;

use crate::chunks::*;


#[derive(Clone, Debug, Default)]
pub struct Function {
    name:String,
    arity:usize,
    pub chunk:Rc<Chunk>
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}


impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            return write!(f, "<script>");
        } else {
            return write!(f, "<fn {}>", self.name);
        }      
    }
}

impl Function{
    pub fn new<T:ToString>(name:T, arity:usize, chunk: &Rc<Chunk>) -> Self {
        Self {
            name:name.to_string(),
            arity,
            chunk: Rc::clone(chunk)
        }
    }

    pub fn get_chunk(&self) -> Rc<Chunk> {
        Rc::clone(&self.chunk)
    }

    pub fn toplevel(chunk: &Rc<Chunk>) -> Self {
        Self {
            name:"".to_string(),
            arity:0,
            chunk: Rc::clone(chunk)
        }
    }

    /*
    pub fn write(&self, byte:u8, line:usize) {
        self.chunk.borrow_mut().write(byte, line);
    }

    pub fn count(&self) -> usize {
        self.chunk.borrow().count()
    }

    pub fn add_constant(&self, value:Value) -> Option<u8> {
        self.chunk.borrow_mut().add_constant(value)
    }

    pub fn write_at(&self, offset:usize, byte:u8) {
        self.chunk.borrow_mut().write_at(offset, byte);
    }

    pub fn disassemble<T:ToString>(&self, name:T)
    where T:Display{
        self.chunk.borrow().disassemble(name);
    }
 */
}