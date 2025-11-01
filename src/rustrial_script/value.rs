//! Value types for RustrialScript

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn as_int(&self) -> Result<i32, &'static str> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err("Expected integer"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Nil => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        self.as_bool()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}
