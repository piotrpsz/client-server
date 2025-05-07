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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub code: i32,
    pub msg: String,
    pub kind: String,
    pub io: bool,
    pub serde: bool
}


impl Error {
    pub fn new(code: i32, msg: &str) -> Self {
        Error {
            code,
            msg: msg.to_string(),
            ..Default::default()
        }
    }
    pub fn with_error_kind(code: i32, kind: ErrorKind, msg: &str) -> Self {
        Error {
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
                code: -2,
                msg: format!("{:?}", e), 
                kind: Other.to_string(), 
                io: false,
                serde: true
            }
        })
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            code: 0,
            msg: "".to_string(),
            kind: Other.to_string(),
            io: false,
            serde: false,
        }   
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error { 
            code: err.raw_os_error().unwrap(),
            msg: err.to_string(),
            kind: err.kind().to_string(),
            io: true,
            ..Default::default()
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error { 
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
