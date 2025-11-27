use std::fmt::Display;
use std::rc::Rc;

use crate::chunks::*;

#[derive(Clone, Debug)]
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
            return write!(f, "{}", self.name);
        }      
    }
}

impl Function{
    pub fn new(chunk: &Rc<Chunk>) -> Self {
        Self {
            name: "".to_string(),
            arity:0,
            chunk: Rc::clone(chunk)
        }
    }

    pub fn get_chunk(&self) -> Rc<Chunk> {
        Rc::clone(&self.chunk)
    }
 
}