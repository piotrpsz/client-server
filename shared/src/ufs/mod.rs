use std::ffi::CStr;
use std::fmt::Display;
use std::io;
use std::io::ErrorKind;
use std::io::ErrorKind::Other;
use serde_json;
use crate::data::answer::Answer;

pub mod dir;
pub mod fileinfo;
pub mod file;


type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub kind: ErrorKind
}

impl Error {
    pub fn from_errno() -> Error {
        unsafe {
            let errno = io::Error::last_os_error().raw_os_error().unwrap();
            let err_str = libc::strerror(errno);
            let cstr = CStr::from_ptr(err_str);
            let message = cstr.to_str().unwrap().to_string();
            Error { code: errno, message: message.into(), kind: Other }
        }
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(Other, err.message)
    }   
}

impl From<Answer> for Error {
    fn from(answer: Answer) -> Error {
        Error { code: answer.code, message: answer.data[0].clone(), kind: Other }
    }  
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error { code: err.raw_os_error().unwrap(), message: err.to_string(), kind: err.kind() }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error { code: -1, message: err.to_string(), kind: Other }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            -1  => write!(f, "Error: {}", self.message),    
            _ => write!(f, "Error: {} [{}: {}]", self.message, self.code, self.kind)
        }
    }   
}

