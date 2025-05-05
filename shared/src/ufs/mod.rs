use std::option::Option;
use std::ffi::CStr;
use std::fmt::Display;
use std::io;
use std::io::ErrorKind::Other;
use serde_json;

pub mod dir;
pub mod fileinfo;
pub mod file;


type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub kind: Option<io::ErrorKind>
}

impl Error {
    pub fn from_errno() -> Error {
        unsafe {
            let errno = io::Error::last_os_error().raw_os_error().unwrap();
            let err_str = libc::strerror(errno);
            let cstr = CStr::from_ptr(err_str);
            let message = cstr.to_str().unwrap().to_string();
            Error { code: errno, message: message.into(), kind: None }
        }
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(Other, err.message)
    }   
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error { code: err.raw_os_error().unwrap(), message: err.to_string(), kind: Some(err.kind()) }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error { code: -1, message: err.to_string(), kind: None }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            Some(kind) => write!(f, "Error: {{ kind: {}, code: {}, message: {} }}", kind, self.code, self.message),
            None => write!(f, "Error: {{ code: {}, message: {} }}", self.code, self.message)       
        }
    }   
}

