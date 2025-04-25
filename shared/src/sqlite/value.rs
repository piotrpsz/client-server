#![allow(dead_code)]
#![allow(unused_imports)]
use std::mem;
use chrono::NaiveDateTime;
use super::types::{ ValueType, Timestamp };

static U32_SIZE: usize = size_of::<u32>();
static I64_SIZE: usize = size_of::<i64>();
static F64_SIZE: usize = size_of::<f64>();

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    pub fn kind(&self) -> ValueType {
        match self {  
            Value::Null => ValueType::Null,
            Value::Integer(_) => ValueType::Integer,
            Value::Real(_) => ValueType::Real,
            Value::Text(_) => ValueType::Text,
            Value::Blob(_) => ValueType::Blob,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Value::Null => 0,
            Value::Integer(_) => I64_SIZE,
            Value::Real(_) => F64_SIZE,
            Value::Text(v) => v.len(),
            Value::Blob(v) => v.len(),
        }
    }
    
    // Konwersja wartości do ciągu bajtów.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            // W poniższych gałęziach bajty wartości poprzedzone są
            // markerem (jeden bajt) typu wartości.
            Value::Null => {
                vec![ValueType::Null as u8]
            },
            Value::Integer(v) => {
                let mut data = v.to_be_bytes().to_vec();
                let mut buffer = Vec::with_capacity(data.len() + 1);
                buffer.push(ValueType::Integer as u8);
                buffer.append(&mut data);
                buffer
            },
            Value::Real(v) => {
                let mut data = v.to_be_bytes().to_vec();
                let mut buffer = Vec::with_capacity(data.len() + 1);
                buffer.push(ValueType::Real as u8);
                buffer.append(&mut data);
                buffer
            },
            // W poniższych gałęziach bajty wartości poprzedzone są
            // nie tylko markerem typu, ale również liczbą u32 określającą
            // liczbę bajtów w kontenerze.
            Value::Text(v) => {
                let mut data = v.as_bytes().to_vec();
                let mut buffer = Vec::with_capacity(data.len() + 1 + U32_SIZE);
                buffer.push(ValueType::Text as u8);
                buffer.append(&mut (data.len() as u32).to_be_bytes().to_vec());
                buffer.append(&mut data);
                buffer
            },
            Value::Blob(v) => {
                let mut data = v.clone();
                let mut buffer = Vec::with_capacity(data.len() + 1 + U32_SIZE);
                buffer.push(ValueType::Blob as u8);
                buffer.append(&mut (data.len() as u32).to_be_bytes().to_vec());
                buffer.append(&mut data);
                buffer   
            },
        }
    }
    pub fn from_bytes(data: &[u8]) -> Option<Value> {
        let size = data.len();
        if size == 0 {
            return None;
        }
        
        match ValueType::value_type(data[0]) {     // pierwszy bajt danych jest markerem typu
            ValueType::Null => {
                Some(Value::Null)
            }
            ValueType::Integer => {
                Some(Value::Integer(i64::from_be_bytes(data[1..].try_into().unwrap())))
            }
            ValueType::Real => {
                Some(Value::Real(f64::from_be_bytes(data[1..].try_into().unwrap())))
            }
            ValueType::Text => {
                let len = u32::from_be_bytes(data[1..5].try_into().unwrap());
                let text = String::from_utf8(data[5..len as usize].to_vec()).unwrap();
                Some(Value::Text(text))
            }
            ValueType::Blob => {
                let len = u32::from_be_bytes(data[1..5].try_into().unwrap());
                let blob = data[5..len as usize].to_vec();
                Some(Value::Blob(blob))
            }
        }
    }
}

impl From<Timestamp> for Value {
    fn from(tms: Timestamp) -> Self {
        Value::Integer(tms.value())
    }
}

impl From<NaiveDateTime> for Value {
    fn from(dt: NaiveDateTime) -> Self {
        Value::Text(dt.format("%Y-%m-%d %H:%M:%S").to_string())   
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Integer(i64::from(v))
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::Integer(i64::from(v))
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(i64::from(v))
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(i64::from(v))
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Real(f64::from(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Integer(if v { 1 } else { 0 })
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Blob(v)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Blob(v.to_vec())
    }
}

// Value --> x

impl From<Value> for i64 {
    fn from(v: Value) -> Self {
        match v {
            Value::Integer(v) => v,
            _ => panic!("Value::Integer expected"),
        }
    }
}

impl From<Value> for f64 {
    fn from(v: Value) -> Self {
        match v {
            Value::Real(v) => v,
            _ => panic!("Value::Real expected"),
        }
    }
}

impl From<Value> for String {
    fn from(v: Value) -> Self {
        match v {
            Value::Text(v) => v,
            _ => panic!("Value::Text expected"),
        }
    }
}

impl From<Value> for Vec<u8> {
    fn from(v: Value) -> Self {
        match v {
            Value::Blob(v) => v,
            _ => panic!("Value::Blob expected"),
        }
    }
}

impl From<Value> for Timestamp {
    fn from(v: Value) -> Self {
        Timestamp::from(i64::from(v))
    }
}

impl From<Value> for NaiveDateTime {
    fn from(v: Value) -> Self {
        NaiveDateTime::parse_from_str(&String::from(v), "%Y-%m-%d %H:%M:%S").unwrap()
    }
}
