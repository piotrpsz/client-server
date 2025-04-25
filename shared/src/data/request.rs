#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use serde_json::Result;
use std::fmt::Debug;

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub command: String,
    pub params: Vec<String>
}

impl Request {
    pub fn new(command: String, params: Vec<String>) -> Self {
        Self {
            command,
            params
        }
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
    pub fn from_json(json: &[u8]) -> Result<Self> {
        serde_json::from_str(String::from_utf8_lossy(json).as_ref())
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request {{ command: {}, params: {:?} }}", self.command, self.params)
    }
}
