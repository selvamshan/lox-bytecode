use std::fmt::Display;
use std::ops::{Add, Sub, Mul, Div, Neg};


#[derive(Clone, Copy)]
pub enum  Value {
    Boolean(bool),
    Number(f64),
    Nil
}

//pub type Value = f64;

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match  self {
            Value::Boolean(t) => write!(f, "{t}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "nil")
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number( a + b),
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



pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value:Value) -> usize {
        let count = self.values.len();
        self.values.push(value);
        count
    }

    pub fn free(&mut self) {
        self.values =Vec::new();
    }

    pub fn print_value(&self, which: usize) {
        print!("{}", self.values[which])
    }

    pub fn read_value(&self, which: usize) -> Value {
        self.values[which]
    }
}