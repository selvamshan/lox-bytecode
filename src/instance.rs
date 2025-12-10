use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use::std::fmt::{Display, Result, Formatter};

use crate::value::*;
use crate::class::*;


#[derive(Debug)]
pub struct Instance {
    klass: Rc<Class>,
    fields: RefCell<HashMap<String, Value>>,
}


impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self { 
            klass: Rc::clone(&klass), 
            fields: RefCell::new(HashMap::new())
        }
    }

    pub fn get_field(&self, field_name:&String) -> Option<Value> {
        self.fields.borrow().get(field_name).cloned()
    }

    pub fn set_field<T:Into<String>>(&self, field_name:T, value:&Value) {
         self.fields.borrow_mut().insert(field_name.into(), value.clone());
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} instacne", self.klass)
    }
}