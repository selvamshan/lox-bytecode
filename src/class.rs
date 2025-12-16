use std::fmt::Display;
use std::fmt::Result;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::collections::HashMap;

use crate::value::*;
use crate::closure::*;



#[derive(Debug)]
pub struct Class{
    name: String,   
    methods: RefCell<HashMap<String, Rc<Closure>>>,
    init: RefCell<Option<Rc<Closure>>>
}


impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

impl Class {
    pub fn new(name:String) -> Self {
        Self {
            name,
            methods: RefCell::new(HashMap::new()),
            init: RefCell::new(None)
        }
    }

    pub fn set_init_method(&self, closure:Rc<Closure>) {      
        self.init.replace(Some(closure));
    }

    pub fn get_init_method(&self) -> Option<Rc<Closure>> {
        self.init.borrow().deref().clone()
    }

    pub fn add_method<T:Into<String>>(&self, name:T, value:&Value) {
        if let Value::Closure(closure) = value{
         self.methods.borrow_mut().insert(name.into(), Rc::clone(closure));
        }
    }

    pub fn get_mehtod(&self, name:&str)-> Option<Rc<Closure>> {
        self.methods.borrow().get(name).cloned()
    }

    pub fn copy_method(&self, superclass:&Self) {
        for (k, v) in superclass.methods.borrow().iter() {
            self.methods.borrow_mut().insert(k.to_string(), Rc::clone(v));
        }
    }
}