use std::rc::Rc;
use std::fmt::{Display, Result};

use crate::function::*;
use crate::chunks::*;


#[derive(Debug)]
pub struct Closure{
    function: Rc<Function>
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        self.function.fmt(f)
    }
}

impl Closure {
    pub fn new(function:Rc<Function>) -> Self {
        Self{
            function: Rc::clone(&function)
        }
    }

    pub fn arity(&self) -> usize {
        self.function.arity()
    }

    
    pub fn get_chunk(&self) -> Rc<Chunk> {
        self.function.get_chunk()
    }

    pub fn stack_name(&self) -> &str {
        self.function.stack_name()
    }

}