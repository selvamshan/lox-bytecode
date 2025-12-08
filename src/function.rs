use std::fmt::Display;
use std::rc::Rc;

use crate::chunks::*;


#[derive(Debug, Default)]
pub struct Function {
    name:String,
    arity:usize,
    pub chunk:Rc<Chunk>,
    upvalue_count:usize,
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        panic!("Comparing the ord of the two funtions")
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function { 
            name: self.name.clone(),
            arity: self.arity,
             chunk: self.chunk.clone(), 
             upvalue_count: self.upvalue_count
        }
    }
}


impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }      
    }
}

impl Function{
    pub fn new<T:Into<String>>(
        name:T, arity:usize, chunk: &Rc<Chunk>, upvalue_count:usize) -> Self {
        Self {
            name:name.into(),
            arity,
            chunk: Rc::clone(chunk),
            upvalue_count
        }
    }

    pub fn get_chunk(&self) -> Rc<Chunk> {
        Rc::clone(&self.chunk)
    }

    pub fn toplevel(chunk: &Rc<Chunk>) -> Self {
        Self {
            name:"".to_string(),
            arity:0,
            chunk: Rc::clone(chunk),
            upvalue_count:0
        }
    }
  
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn stack_name(&self) -> &str {
        if self.name.is_empty(){
            "script"
        } else {
            self.name.as_str()
        }
    }

    pub fn upvalue(&self) -> usize {
        self.upvalue_count
    }
}