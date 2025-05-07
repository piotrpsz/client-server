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
#![allow(unused_imports)]

use std::{
    collections::HashMap,
    convert::From,
    fmt::{ self, Display, Debug },
};
use super::value::Value;
use chrono::Local;

/********************************************************************
 *                                                                  *
 *                             R o w                                *
 *                                                                  *
 *******************************************************************/

pub type Row = HashMap<String, Value>;

/********************************************************************
 *                                                                  *
 *                      S Q L i t e E r r o r                       *
 *                                                                  *
 *******************************************************************/

pub struct SQLiteError {
    pub message: String,
    pub code: i32,
}

impl SQLiteError {
    pub fn new(code: i32, message: String) -> Self {
        Self { message, code }   
    }
}

impl Debug for SQLiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SQLiteError {{ data: {}, code: {} }}", self.message, self.code)
    }
}

/********************************************************************
 *                                                                  *
 *                       V a l u e T y p e                          *
 *                                                                  *
 *******************************************************************/

pub enum ValueType {
    Null,
    Integer,
    Real,
    Text,
    Blob,
}

impl ValueType {
    pub fn value_type(v: u8) -> Self {
        match v {
            1 => Self::Integer,
            2 => Self::Real,
            3 => Self::Text,
            4 => Self::Blob,
            _ => Self::Null,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => write!(f, "INT"),
            Self::Real => write!(f, "REAL"),
            Self::Text => write!(f, "TEXT"),
            Self::Blob => write!(f, "BLOB"),
            _ => write!(f, "NULL"),       
        }
    }
}

/********************************************************************
 *                                                                  *
 *                       T i m e s t a m p                          *
 *                                                                  *
 *******************************************************************/

#[derive(Clone, Copy)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(Local::now().timestamp())
    }
    pub fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl From<i64> for Timestamp {
    fn from(timestamp: i64) -> Self {
        Self(timestamp)
    }
}