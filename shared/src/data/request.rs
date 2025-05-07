// MIT License
// 
// Copyright (c) 2025 Piotr Pszczółkowski
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use serde_json::Result;
use std::fmt::Debug;
use std::time::{ SystemTime, UNIX_EPOCH };

#[derive(Serialize, Deserialize, Clone)]
pub struct Request {
    id: u64,
    timestamp: u64,
    pub command: String,
    pub params: Vec<String>
}

impl Request {
    pub fn new(command: String, params: Vec<String>) -> Self {
        Self {
            id: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            command,
            params
        }
    }
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
    pub fn id(&self) -> u64 {
        self.id
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

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request {{ command: {}, params: {:?} }}", self.command, self.params)
    }
}
