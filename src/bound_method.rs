use std::rc::Rc;
use std::fmt::{Display, Result, Formatter};

use crate::closure::*;
use crate::value::*;

#[derive(Debug)]
pub struct BoundMethod {
    receiver: Value,
    method: Rc<Closure>
}

impl BoundMethod {
    pub fn new(receiver: &Value, method: &Rc<Closure>) -> Self {
        Self{
            receiver:receiver.clone(),
            method: Rc::clone(method)
        }
    }

    pub fn get_closure(&self) -> Rc<Closure> {
        Rc::clone(&self.method)
    }

    pub fn get_recevier(&self) -> Value {
        self.receiver.clone()
    }
    
}

impl Display for BoundMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
       self.method.fmt(f)
    }
}