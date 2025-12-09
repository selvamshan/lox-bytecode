use std::fmt::Display;
use std::fmt::Result;


#[derive(Debug)]
pub struct Class{
    name: String
}


impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

impl Class {
    pub fn new(name:String) -> Self {
        Self {
            name
        }
    }
}