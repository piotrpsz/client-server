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
use Default;
use std::ffi::CStr;
use std::io:: {
    self,
    ErrorKind::{
        self,
        Other
    },
};
use serde::{Deserialize, Serialize};
use crate::data::answer::Answer;
use crate::xerror::ErrSrc::{App, Unknown};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum ErrSrc {
    Unknown,
    IO,
    Errno,
    Serde,
    App
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub src: ErrSrc,
    pub code: i32,
    pub msg: String,
    pub kind: String,
}

impl Error {
    pub fn new(code: i32, msg: &str) -> Self {
        Error {
            src: App,
            code,
            msg: msg.to_string(),
            ..Default::default()
        }
    }
    pub fn with_error_kind(code: i32, kind: ErrorKind, msg: &str) -> Self {
        Error {
            src: App,
            code,
            msg: msg.to_string(),
            kind: kind.to_string(),
            ..Default::default()
        }
    }
    
    pub fn from_errno() -> Self {
        unsafe {
            let errno = io::Error::last_os_error().raw_os_error().unwrap();
            let err_str = libc::strerror(errno);
            let cstr = CStr::from_ptr(err_str);
            let message = cstr.to_str().unwrap().to_string();
            Error {
                src: ErrSrc::Errno,
                code: errno,
                msg: message.into(),
                ..Default::default()
            }
        }
    }
    
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            Error { 
                src: ErrSrc::Serde,
                code: -2,
                msg: format!("{:?}", e), 
                kind: Other.to_string(), 
            }
        })
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            src: Unknown,
            code: 0,
            msg: "".to_string(),
            kind: Other.to_string(),
        }   
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            src: ErrSrc::IO,
            code: err.raw_os_error().unwrap(),
            msg: err.to_string(),
            kind: err.kind().to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error {
            src: ErrSrc::Serde,
            code: -1,
            msg: format!("{}", err),
            ..Default::default()
        }
    }   
}

impl From<Error> for Answer {
    fn from(err: Error) -> Self {
        Answer::new(err.code, err.to_json().as_str(), "")
    }
}

impl From<Answer> for Error {
    fn from(answer: Answer) -> Self {
        Error::from_json(&answer.message).unwrap()
    }
}
