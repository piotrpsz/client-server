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
extern crate libc;
use std::string::String;
use std::ffi::CString;
use std::fs::OpenOptions;
use chrono::{DateTime, Local};
use libc::{c_uint, mode_t};
use crate::xerror::{Result, Error};

static FILE_ERR_CODE: i32 = -1;
static FILE_ALREADY_OPENED: &str = "file is already opened";
static FILE_NOT_OPENED: &str = "file is not opened";
static FILE_READ_ERROR: &str = "read bytes is not equal to buffer size";
static FILE_WRITE_ERROR: &str = "write bytes is not equal to buffer size";

#[repr(i32)]
pub enum FLAG {
    Read = libc::O_RDONLY,
    Write = libc::O_WRONLY,
    ReadWrite = libc::O_RDWR,
}

pub struct File {
    fd: i32,
    path: String,
    cstr: CString,
}

impl File {
    pub fn new(path: &str) -> Self {
        Self {
            fd: -1,
            path: path.into(),
            cstr: CString::new(path).unwrap(),
        }
    }
    
    #[inline]
    pub fn exist(&self) -> Result<()> {
        self.access(libc::F_OK)
    }
    #[inline]
    pub fn readable(&self) -> Result<()> {
        self.access(libc::R_OK)
    }
    #[inline]
    pub fn writable(&self) -> Result<()> {
        self.access(libc::W_OK)
    }
    #[inline]
    pub fn executable(&self) -> Result<()> {
        self.access(libc::X_OK)
    }
    #[inline]
    fn access(&self, flag: i32) -> Result<()> {
        unsafe {
            match libc::access(self.cstr.as_ptr(), flag) {
                0 => Ok(()),
                _ => Err(Error::from_errno())
            }
        }
    }
    
    /// Utworzenie nowego pustego pliku.
    pub fn touch(&self) -> Result<()> {
        let stat = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.path.as_str()); 
        match  stat {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from(err))
        }
    }
    
    /// Zmiana praw dostępu do pliku.
    pub fn chmod(&self, mode: u32) -> Result<()> {
        unsafe {
            match libc::chmod(self.cstr.as_ptr(), mode as mode_t) {
                0 => Ok(()),
                _ => Err(Error::from_errno())
            }
        }
    }
    
    pub fn utime(&self, dt: DateTime<Local>) -> Result<()>{
        let utb = libc::utimbuf {
            actime: dt.timestamp(),
            modtime: dt.timestamp(),
        };
        unsafe {
            match libc::utime(self.cstr.as_ptr(), &utb) {
                0 => Ok(()),
                _ => Err(Error::from_errno())
            }
        }
    }
    
    /// Usunięcie pliku.
    /// Przed usunięciem plik zostanie zamknięty.
    pub fn rm(&mut self) -> Result<()> {
        unsafe {
            self.close()?;
            match libc::unlink(self.cstr.as_ptr()) {
                0 => Ok(()),
                _ => Err(Error::from_errno())
            }
        }
    }
    
    /// Zmiana nazwy pliku.
    /// Jeśli wszystko się uda, aktualny obiekt zawiera nową nazwę. 
    pub fn rename(&mut self, dst_str: &str) -> Result<()> {
        let dst = CString::new(dst_str).unwrap();
        unsafe {
            match libc::rename(self.cstr.as_ptr(), dst.as_ptr()) {
                0 => {
                    self.path = dst_str.into();
                    self.cstr = dst;
                    Ok(())
                },
                _ => Err(Error::from_errno())
            }
        }
    }

    
    pub fn open(&mut self) -> Result<()> {
        self.open_with_flag(FLAG::ReadWrite)
    }
    pub fn open_read_only(&mut self) -> Result<()> {
        self.open_with_flag(FLAG::Read)
    }
    pub fn open_write_only(&mut self) -> Result<()> {
        self.open_with_flag(FLAG::Write)
    }
    
    fn already_opened(&self) -> Result<()> {
        match self.fd {
            -1 => Ok(()),
            _ => Err(Error::new(FILE_ERR_CODE, FILE_ALREADY_OPENED)),
        }
    }
    fn not_opened(&self) -> Result<()> {
        match self.fd {
            -1 => Err(Error::new(FILE_ERR_CODE, FILE_NOT_OPENED)),
            _ => Ok(()),
        }
    }
    
    /// Otwarcie istniejącego pliku ze wskazanie flag.
    pub fn open_with_flag(&mut self, flag: FLAG) -> Result<()> {
        self.already_opened()?;
        
        unsafe {
            match libc::open(self.cstr.as_ptr(), flag as i32) {
                -1 => Err(Error::from_errno()),
                fd => {
                    self.fd = fd; 
                    Ok(())
                }
            }
        }
    } // end of 'open_with_flag'
    
    /// Utworzenie pliku do zapisu i odczytu.
    /// Jeśli plik już istnieje, zostanie zwrócony błąd.
    pub fn create(&mut self) -> Result<()> {
        self.already_opened()?;
        
        unsafe {
            let flags = libc::O_CREAT | libc::O_EXCL | libc::O_RDWR;
            let mode = libc::S_IRWXU | libc::S_IRWXG | libc::S_IROTH;
            match libc::open(self.cstr.as_ptr(), flags, mode as c_uint) {
                -1 => Err(Error::from_errno()),
                fd => {
                    self.fd = fd;
                    Ok(())
                }
            }
        }
    } // end of 'create'
    
    /// Zamknięcie pliku (jeśli jest otwarty).
    pub fn close(&mut self) -> Result<()> {
        if self.fd == -1 {
            return Ok(());
        }
        unsafe {
            match libc::close(self.fd) {
                0 => {
                    self.fd = -1;
                    Ok(())
                },
                _ => Err(Error::from_errno())
            }
        }
    } // end of 'clone'
    
    /// Wyznaczenie liczby bajtów otwartego pliku.
    pub fn size(&self) -> Result<usize> {
        self.not_opened()?;
        
        let size = self.seek_end()? - self.seek_begin()?;
        Ok(size)
    }
    
    /// Odczyt bajtów do wektora.
    /// Czytamy dokładnie tyle bajtów, ile zmieści się do tego wektora.
    /// Odczyt zaczyna się na aktualnej pozycji kursora pliku.
    pub fn read_exact(&self, buffer: &mut [u8]) -> Result<()> {
        self.not_opened()?;

        unsafe {
            match libc::read(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len()) {
                -1 => Err(Error::from_errno()),
                nbytes => {
                    if nbytes as usize != buffer.len() {
                        return Err(Error::new(FILE_ERR_CODE, FILE_READ_ERROR ));
                    }
                    Ok(())
                }
            }
        }
    }
    
    /// Odczyt całego pliku do wektora bajtów.
    pub fn read_all_vec(&self) -> Result<Vec<u8>> {
        self.not_opened()?;

        // Wyznaczenie liczby bajtów w pliku, Kursor pliku jest na początku pliku.
        let nbytes = self.size()?;
        let mut buffer = Vec::with_capacity(nbytes);
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }
    
    /// Odczyt całego pliku jako tekstu.
    pub fn read_all_str(&self) -> Result<String> {
        self.not_opened()?;
        
        let data = self.read_all_vec()?;
        String::from_utf8(data)
            .map_err(|e| Error::new(FILE_ERR_CODE, e.to_string().as_str()))
    }

    /// Zapis przysłanych bajtów do pliku.
    /// Zapis następuje na aktualnej pozycji kursora pliku.
    pub fn write(&self, buffer: &[u8]) -> Result<()> {
        self.not_opened()?;
        
        unsafe {
            match libc::write(self.fd, buffer.as_ptr() as *const libc::c_void, buffer.len()) { 
                -1 => Err(Error::from_errno()),
                nbytes => {
                    if nbytes as usize != buffer.len() {
                        return Err(Error::new(FILE_ERR_CODE, FILE_WRITE_ERROR));
                    }
                    Ok(())
                }
            }
        }
    }
    
    /// Zapis linii tekstu do pliku.
    /// Jeśli linia nie kończy się znakiem nowej linii, to zostanie on dodany.
    /// Zapis następuje na aktualnej pozycji kursora pliku.
    pub fn write_line(&self, text: &str) -> Result<()> {
        self.not_opened()?;

        let mut buffer = text.as_bytes().to_vec();
        match buffer.last() {
            Some(b'\n') => (),
            _ => buffer.push(b'\n'),
        }
        self.write(&buffer)?;
        Ok(())
    }
    
    #[inline]
    fn seek_begin(&self) -> Result<usize> {
        self.seek_to(libc::SEEK_SET)
    }
    #[inline]
    fn seek_end(&self) -> Result<usize> {
        self.seek_to(libc::SEEK_END)
    }
    #[inline]
    fn seek_current(&self) -> Result<usize> {
        self.seek_to(libc::SEEK_CUR)   
    }
    #[inline]
    fn seek_to(&self, whence: i32) -> Result<usize> {
        self.not_opened()?;
        
        unsafe {
            match  libc::lseek(self.fd, 0, whence) {
                -1 => Err(Error::from_errno()),
                offset => Ok(offset as usize),
            }
        }
    }
}  // 

/********************************************************************
*                                                                   *
*              S T A T I C   F U N C T I O N S                      *
*                                                                   *
********************************************************************/

pub fn exist(path: &str) -> Result<()> {
    let cpath = CString::new(path).unwrap();
    unsafe {
        match libc::access(cpath.as_ptr(), libc::F_OK) {
            0 => Ok(()),
            _ => Err(Error::from_errno())
        }
    }
}

pub fn rename(from: &str, to: &str) -> Result<()> {
    unsafe {
        let from = CString::new(from).unwrap();
        let to = CString::new(to).unwrap();
        match libc::rename(from.as_ptr(), to.as_ptr()) {
            0 =>  Ok(()),
            _ => Err(Error::from_errno())
        }
    }
}

/// Usunięcie pliku.
/// Przed usunięciem plik zostanie zamknięty.
pub fn rm(path: &str) -> Result<()> {
    unsafe {
        let cpath = CString::new(path).unwrap();
        match libc::unlink(cpath.as_ptr()) {
            0 => Ok(()),
            _ => Err(Error::from_errno())
        }
    }
}

/// Utworzenie nowego pustego pliku.
pub fn touch(path: &str) -> Result<()> {
    let stat = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path);
    match  stat {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::from(err))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.fd != -1 {
            let _ = self.close();
        }
    }
}
