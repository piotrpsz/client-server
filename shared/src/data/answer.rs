#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use serde_json::Result;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct Answer {
    id: u64,
    timestamp: u64,
    pub code: i32,
    pub message: String,
    pub data: Vec<String>
}

impl Answer {
    pub fn new(code: i32, message: String) -> Self {
        Self {
            id: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            code,
            message,
            data: Vec::new()
        }
    }
    pub fn new_with_data(code: i32, message: String, data: Vec<String>) -> Self {
        Self {
            id: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            code,
            message,
            data
        }
    }
    
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    
    pub fn add(&mut self, data: String) {
        self.data.push(data);
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
    pub fn from_json(json: &[u8]) -> Result<Self> {
        serde_json::from_str(String::from_utf8_lossy(json).as_ref())
    }
}

impl Debug for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Answer {{ code: {}, message: {}, data: {:?} }}", self.code, self.message, self.data)
    }
}
