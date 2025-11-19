use std::fmt::Display;
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::usize;



#[derive(PartialEq, PartialOrd)]
pub enum  Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Str(String)
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Number(n) => Value::Number(*n),
            Value::Nil => Value::Nil,
            Value::Str(s) => Value::Str(s.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match  self {
            Value::Boolean(t) => write!(f, "{t}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "nil"),
            Value::Str(s) => write!(f, "{s}")
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number( a + b),
            (Value::Str(a), Value::Str(b)) => Value::Str( a + &b),
            _ => panic!("Invalid operations")

        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number( a - b),
            _ => panic!("Invalid operations")

        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number( a * b),
            _ => panic!("Invalid operations")

        }
    }
}


impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number( a / b),
            _ => panic!("Invalid operations")

        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        match self {
            Value::Number(a) => Value::Number(-a),
            _ => panic!("Invalid operations")
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
      matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Str(_))
    }

    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }
}


pub struct ValueArray {
    values: Vec<Value>    
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
             values: Vec::new(),            
         }
    }

    pub fn write(&mut self, value:Value) -> usize {
        /* String probing 
        if let Value::Str(s) = value.clone() {
            for (i, v) in self.values .iter().enumerate() {
                if let Value::Str(existing) = v {
                    if existing == &s {
                        return i;
                    }
                }
            }
        } 
        */
        let count = self.values.len();
        self.values.push(value);
        count
    }

    
    pub fn print_value(&self, which: usize) {
        print!("{}", self.values[which])
    }

    pub fn read_value(&self, which: usize) -> &Value {
        &self.values[which]
    }

  
}