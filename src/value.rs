use core::fmt::Display;
use std::cmp::{PartialEq, PartialOrd};
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Sub};

use crate::object::ObjFunction;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f64),
    Integer(i64),
    String(String),
    None,
    True,
    ObjFunction(ObjFunction),
    False,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::True => true,
            Value::Integer(i) => *i != 0,
            Value::Float(i) => *i != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Float(_) | Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn type_of(&self) -> String {
        match self {
            Value::Float(_) => "float".to_owned(),
            Value::Integer(_) => "int".to_owned(),
            Value::True => "bool".to_owned(),
            Value::False => "bool".to_owned(),
            Value::String(_) => "string".to_owned(),
            Value::ObjFunction(_) => "function".to_owned(),
            Value::None => "none".to_owned(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Float(n) => write!(f, "{}", n),
            Value::Integer(n) => write!(f, "{}", n),
            Value::True => write!(f, "true"),
            Value::False => write!(f, "false"),
            Value::ObjFunction(n) => write!(f, "{}", n),
            Value::None => write!(f, "none"),
        }
    }
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, other: Value) -> Result<Value, String> {
        let type_self = self.type_of();
        let type_other = other.type_of();
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(format!(
                "Unsupported add operation on types {} and {}",
                type_self, type_other
            )
            .to_owned()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, other: Value) -> Result<Value, String> {
        let type_self = self.type_of();
        let type_other = other.type_of();
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            _ => Err(format!(
                "Unsupported substract operation on types {} and {}",
                type_self, type_other
            )
            .to_owned()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, other: Value) -> Result<Value, String> {
        let type_self = self.type_of();
        let type_other = other.type_of();
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Integer(a), Value::String(b)) => Ok(Value::String(b.repeat(a as usize))),
            (Value::String(a), Value::Integer(b)) => Ok(Value::String(a.repeat(b as usize))),
            _ => Err(format!(
                "Unsupported multiply operation on types {} and {}",
                type_self, type_other
            )
            .to_owned()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, other: Value) -> Result<Value, String> {
        let type_self = self.type_of();
        let type_other = other.type_of();
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float(a as f64 / b as f64)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / b as f64)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
            _ => Err(format!(
                "Unsupported divide operation on types {} and {}",
                type_self, type_other
            )
            .to_owned()),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Value {
        match self {
            Value::Float(a) => Value::Float(-a),
            Value::Integer(a) => Value::Integer(-a),
            _ => panic!("Unsupported operation"),
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, String>;

    fn bitand(self, other: Value) -> Result<Value, String> {
        let ret = self.is_truthy() && other.is_truthy();
        if ret {
            return Ok(Value::True);
        } else {
            return Ok(Value::False);
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, String>;

    fn bitor(self, other: Value) -> Result<Value, String> {
        let ret = self.is_truthy() || other.is_truthy();
        if ret {
            return Ok(Value::True);
        } else {
            return Ok(Value::False);
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Value {
        if self.is_truthy() {
            return Value::False;
        } else {
            return Value::True;
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::True, Value::True) => true,
            (Value::False, Value::False) => true,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Integer(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            _ => None,
        }
    }
}

pub fn print_value(value: Value) {
    print!("{}", value);
}
