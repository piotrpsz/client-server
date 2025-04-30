extern crate libc;
use std::ffi::CString;
use std::io::Error;
use std::os;
use libc::c_int;

// #[macro_export]
macro_rules! fpos {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let fname = type_name_of(f);
        let name = match &fname[..fname.len() - 3].rfind(':') {
            Some(idx) => &fname[idx + 1..fname.len() - 3],
            _ => &fname[..fname.len() - 3],
        };
        format!("[{}::{}:{}] Error", file!(), name, line!())
    }};
}

// pub const O_RDONLY: c_int = 0;
// pub const O_WRONLY: c_int = 1;
// pub const O_RDWR: c_int = 2;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum OpenMode {
    ReadOnly = 0,
    WriteOnly = 1,
    ReadWrite = 2,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum FileType {
    RegularFile,
    Directory,
    CharacterDevice,
    BlockDevice,
    PipeOrFIFO,
    Socket,
    SymbolicLink,
}

#[derive(Debug, Default)]
pub struct File {
    fd: i32,
    fpath: String,
}

impl File {
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            fd: -1,
            fpath: path.as_ref().to_string(),
        }
    }

    pub fn open(&mut self, mode: OpenMode) -> bool {
        if self.fd == -1 {
            eprintln!("file already opened ({})", fpos!());
            return false;
        }
        unsafe {
            let cstr = CString::new(self.fpath.clone()).unwrap();
            self.fd = libc::open(cstr.as_ptr(), mode as c_int);
            match self.fd {
                -1 => {
                    print_error(&fpos!());
                    false                    
                },
                _ => true,
            }
        }
    }   // fn open

    pub fn close(&mut self) {
        if self.fd == -1 {
            eprintln!("file already closed ({})", fpos!());
            return;
        }
        unsafe {
            libc::close(self.fd);
            self.fd = -1;
        }
    }
    
    
    
} // impl File

fn print_error(info: &str) {
    let errno = Error::last_os_error().raw_os_error().unwrap();
    let err_str = unsafe { libc::strerror(errno) };
    let cstr = unsafe { CString::from_raw(err_str as *mut os::raw::c_char) };
    println!("Error: {} ({})", cstr.to_str().unwrap(), info);
}

pub fn exists(path: &str) -> bool {
    let cstr = CString::new(path).unwrap();
    unsafe { matches!(libc::access(cstr.as_ptr(), libc::F_OK), 0) }
}

pub fn readable(path: &str) -> bool {
    let cstr = CString::new(path).unwrap();
    unsafe { matches!(libc::access(cstr.as_ptr(), libc::R_OK), 0) }
}

pub fn writable(path: &str) -> bool {
    let cstr = CString::new(path).unwrap();
    unsafe { matches!(libc::access(cstr.as_ptr(), libc::W_OK), 0) }
}

pub fn executable(path: &str) -> bool {
    let cstr = CString::new(path).unwrap();
    unsafe { matches!(libc::access(cstr.as_ptr(), libc::X_OK), 0) }
}

pub fn remove(path: &str) -> bool {
    let cstr = CString::new(path).unwrap();
    unsafe { matches!(libc::remove(cstr.as_ptr()), 0) }
}
