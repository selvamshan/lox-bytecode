use std::rc::Rc;
use std::collections::HashMap;
use::std::fmt::{Display, Result, Formatter};

use crate::value::*;
use crate::class::*;


#[derive(Debug)]
pub struct Instance {
    klass: Rc<Class>,
    fields: HashMap<String, Value>,
}


impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self { 
            klass: Rc::clone(&klass), 
            fields: HashMap::new()
        }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} instacne", self.klass)
    }
}