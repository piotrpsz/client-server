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