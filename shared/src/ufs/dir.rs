#![allow(dead_code)]
extern crate libc;

use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use crate::ufs::{Error, Result};
use crate::ufs::fileinfo::FileInfo;



pub struct Dir;

impl Dir {
    /// Odczyt zawartości wskazanego katalogu.
    pub fn read(path: &str, hidden_too: bool) -> Result<Vec<FileInfo>>{
        eprintln!("{}", path);
        
        unsafe {
            let mut files: Vec<FileInfo> = vec![];
            let dirp = Self::opendir(path)?;
            
            loop {
                let dirent = libc::readdir(dirp);
                if dirent.is_null() {
                    libc::closedir(dirp);
                    files.sort_by(|a, b|
                        if a.is_dir() && b.is_dir() {
                            Self::compare_dir_name(&a.name.to_lowercase(), &b.name.to_lowercase())
                        } else if a.is_dir() || b.is_dir() {
                            if a.is_dir() {
                                Ordering::Less
                            }
                            else {
                                Ordering::Greater
                            }
                        }  else {
                            a.name.to_lowercase().cmp(&b.name.to_lowercase())
                        }
                    );
                    return Ok(files);
                }
                let name = CStr::from_ptr((*dirent).d_name.as_ptr()).to_str().unwrap();
                if Self::is_hidden(name) {
                    if hidden_too {
                        files.push(FileInfo::new(name, path)?);
                    } 
                } else{
                    files.push(FileInfo::new(name, path)?);
                }
            }
        }
    }
    
    fn compare_dir_name(a: &str, b: &str) -> Ordering {
        #[derive(PartialOrd, PartialEq)]
        enum Type {
            ThisDir,
            ParentDir,
            Hidden,
            Normal
        }
        
        if a == "." {
            return if b == "." {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
        if b == "." {
            return Ordering::Greater;
        }
        
        if a == ".." {
            return if b == ".." {
                Ordering::Equal
            } else if b == "." {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        if b == ".." {
            return if a == "." {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }

        if Self::is_hidden(a) {
            return if Self::is_hidden(b) {
                a.cmp(b)
            } else {
                Ordering::Less
            }
        }
        if Self::is_hidden(b) {
            return Ordering::Greater
        }
        
        a.cmp(b)
    }
    
    fn is_hidden(name: &str) -> bool {
        match name.len() {
            n if n > 1 => {
                let bytes = name.as_bytes();
                bytes[0] == b'.' && bytes[1] != b'.'
            },
            _ => false
        }
    }
    
    /// Otwarcie do odczytu wskazanego katalogu.
    fn opendir(path: &str) -> Result<*mut libc::DIR> {
        unsafe {
            let c_path = CString::new(path).unwrap();
            match libc::opendir(c_path.as_ptr()) {
                ptr if ptr == null_mut() => Err(Error::from_errno()),
                dirp => Ok(dirp),
            }
        }
    }
    
    pub fn rmdir(path: &str) -> Result<()> {
        unsafe {
            let c_path = CString::new(path).unwrap();
            match libc::rmdir(c_path.as_ptr()) {
                0 => Ok(()),
                _ => Err(Error::from_errno()),
            }
        }
    }
}




