#![allow(dead_code)]
extern crate libc;
use std::string::String;
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io;
use chrono::{DateTime, Local};
use crate::ufs::{Error, Result};

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
                _ => Err(Error::from(io::Error::last_os_error()))
            }
        }
    }
    
    /// Utworzenie nowego pustego pliku.
    pub fn touch(&self) -> Result<()> {
        match OpenOptions::new().create(true).write(true).open(self.path.as_str()) {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from(err))
        }
    }
    
    /// Zmiana praw dostępu do pliku.
    pub fn chmod(&self, mode: u32) -> Result<()> {
        unsafe {
            match libc::chmod(self.cstr.as_ptr(), mode) {
                0 => Ok(()),
                _ => Err(Error::from(io::Error::last_os_error()))
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
                _ => Err(Error::from(io::Error::last_os_error()))
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
                _ => Err(Error::from(io::Error::last_os_error()))
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
                _ => Err(Error::from(io::Error::last_os_error()))
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
    
    /// Otwarcie istniejącego pliku ze wskazanie flag.
    pub fn open_with_flag(&mut self, flag: FLAG) -> Result<()> {
        if self.fd != -1 {
            return Err(Error::from(io::Error::new(io::ErrorKind::Other, "File is already opened")));
        }
        unsafe {
            match libc::open(self.cstr.as_ptr(), flag as i32) {
                -1 => Err(Error::from_errno()),
                fd => {
                    self.fd = fd; 
                    Ok(())
                }
            }
        }
    }

    /// Utworzenie pliku do zapisu i odczytu.
    /// Jeśli plik już istnieje, zostanie zwrócony błąd.
    pub fn create(&mut self) -> Result<()> {
        unsafe {
            let flags = libc::O_CREAT | libc::O_EXCL | libc::O_RDWR;
            let mode = libc::S_IRWXU | libc::S_IRWXG | libc::S_IROTH;
            match libc::open(self.cstr.as_ptr(), flags, mode) {
                -1 => Err(Error::from_errno()),
                fd => {
                    self.fd = fd;
                    Ok(())
                }
            }
        }
    }
    
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
                _ => Err(Error::from(io::Error::last_os_error()))
            }
        }
    }
    
    /// Wyznaczenie liczby bajtów otwartego pliku.
    pub fn size(&self) -> Result<usize> {
        if self.fd == -1 {
            return Err(Error::from(io::Error::new(io::ErrorKind::Other, "File is not opened")));       
        }
        let size = self.seek_end()? - self.seek_begin()?;
        Ok(size)
    }
    
    /// Odczyt bajtów do wektora.
    /// Czytamy dokładnie tyle bajtów, ile zmieści się do tego wektora.
    /// Odczyt zaczyna się na aktualnej pozycji kursora pliku.
    pub fn read_exact(&self, buffer: &mut [u8]) -> Result<()> {
        if self.fd == -1 {
            return Err(Error::from(io::Error::new(io::ErrorKind::Other, "File is not opened")));
        }
        unsafe {
            match libc::read(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len()) {
                -1 => Err(Error::from(io::Error::last_os_error())),
                nbytes => {
                    if nbytes as usize != buffer.len() {
                        return Err(Error::from(io::Error::new(io::ErrorKind::Other, "Read bytes is not equal to buffer size")));
                    }
                    Ok(())
                }
            }
        }
    }
    
    /// Odczyt całego pliku do wektora bajtów.
    pub fn read_all_vec(&self) -> Result<Vec<u8>> {
        if self.fd == -1 {
            return Err(Error::from(io::Error::new(io::ErrorKind::Other, "File is not opened")));
        }
        // Wyznaczenie liczby bajtów w pliku, Kursor pliku jest na początku pliku.
        let nbytes = self.size()?;
        let mut buffer = Vec::with_capacity(nbytes);
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }
    
    /// Odczyt całego pliku jako tekstu.
    pub fn read_all_str(&self) -> Result<String> {
        let data = self.read_all_vec()?;
        String::from_utf8(data).map_err(|_| Error::from(io::Error::new(io::ErrorKind::Other, "Invalid UTF-8 sequence")))
    }

    /// Zapis przysłanych bajtów do pliku.
    /// Zapis następuje na aktualnej pozycji kursora pliku.
    pub fn write(&self, buffer: &[u8]) -> Result<()> {
        unsafe {
            match libc::write(self.fd, buffer.as_ptr() as *const libc::c_void, buffer.len()) { 
                -1 => Err(Error::from(io::Error::last_os_error())),
                nbytes => {
                    if nbytes as usize != buffer.len() {
                        // Niekompletny zapis uznajemy za błąd.
                        return Err(Error::from(io::Error::new(io::ErrorKind::Other, "Write bytes is not equal to buffer size")));
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
        if self.fd == -1 {
            return Err(Error::from(io::Error::new(io::ErrorKind::Other, "File is not opened")));
        }
        if text.is_empty() {
            return Ok(());
        }
        let mut buffer = text.as_bytes().to_vec();
        if *buffer.last().unwrap() != b'\n' {
            buffer.push(b'\n');
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
        unsafe {
            match  libc::lseek(self.fd, 0, whence) {
                -1 => Err(Error::from(io::Error::last_os_error())),
                offset => Ok(offset as usize),
            }
        }
    }
    
}

impl Drop for File {
    fn drop(&mut self) {
        if self.fd != -1 {
            let _ = self.close();
        }
    }
}
