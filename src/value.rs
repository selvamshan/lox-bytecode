use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Result};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

use crate::closure::*;
use crate::function::*;
use crate::class::*;
use crate::instance::*;
use crate::bound_method::*;

pub trait NativeFunc {
    fn call(&self, arg_count: usize, args: &[Rc<RefCell<Value>>]) -> Value;
}

#[derive(Debug)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Str(String),
    Func(Rc<Function>),
    Native(Rc<dyn NativeFunc>),
    Closure(Rc<Closure>),
    Class(Rc<Class>),
    Instance(Rc<Instance>),
    Bound(Rc<BoundMethod>)
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a.eq(b),
            (Value::Number(a), Value::Number(b)) => a.eq(b),
            (Value::Str(a), Value::Str(b)) => a.cmp(b) == Ordering::Equal,
            (Value::Nil, Value::Nil) => true,
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
            (Value::Native(a), Value::Native(b)) => a.type_id() == b.type_id(),
            (Value::Closure(a), Value::Closure(b)) => Rc::ptr_eq(a, b),
            (Value::Bound(a), Value::Bound(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),            
            _ => None,
        }
    }
}

impl Debug for dyn NativeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "<native fn")
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Number(n) => Value::Number(*n),
            Value::Nil => Value::Nil,
            Value::Str(s) => Value::Str(s.clone()),
            Value::Func(f) => Value::Func(Rc::clone(f)),
            Value::Native(f) => Value::Native(Rc::clone(f)),
            Value::Closure(f) => Value::Closure(Rc::clone(f)),
            Value::Class(c) => Value::Class(Rc::clone(c)),
            Value::Instance(i) => Value::Instance(Rc::clone(i)),
            Value::Bound(b) => Value::Bound(Rc::clone(b))
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        match self {
            Value::Boolean(t) => write!(f, "{t}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "nil"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Func(func) => write!(f, "{}", func),
            Value::Native(_) => write!(f, "<native fn>"),
            Value::Closure(c) => write!(f, "{c}"),
            Value::Class(klass) => write!(f, "{}", klass),
            Value::Instance(i) => write!(f, "{i}"),
            Value::Bound(b) => write!(f, "{b}")
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
            _ => panic!("Invalid operations"),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Invalid operations"),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Invalid operations"),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Invalid operations"),
        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        match self {
            Value::Number(a) => Value::Number(-a),
            _ => panic!("Invalid operations"),
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

#[derive(Clone, Debug, Default)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> usize {
        let count = self.values.len();
        self.values.push(value);
        count
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    pub fn print_value(&self, which: usize) {
        print!("{}", self.values[which])
    }

    pub fn read_value(&self, which: usize) -> &Value {
        &self.values[which]
    }
}
