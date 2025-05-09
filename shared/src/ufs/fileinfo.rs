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
use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::ptr::null_mut;
use crate::xerror::{Result, Error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Utc };

const FILEINFO_ERR_CODE:i32 = -2;
const FILEINFO_INVALID_PATH:&str = "invalid path";

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum FileType {
    Unknown,
    RegularFile,
    Directory,
    CharacterDevice,
    BlockDevice,
    PipeOrFIFO,
    Socket,
    SymbolicLink,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,                       // Nazwa pliku (nie ścieżka)
    pub path: String,                       // Kompletna ścieżka do pliku (dir + name)
    owner_id: u32,                          // uid
    owner_name: String,                     // Nazwa właściciela przypisana do uid
    group_id: u32,                          // gid
    group_name: String,                     // Nazwa grupy przypisana do gid
    file_type: FileType,                    // Typ pliku np. Directory, RegularFile itd.
    size: u64,                              // Rozmiar pliku w bajtach
    mode: u32,                              // Oryginalna wartość 'mode' ze 'stat'
    permissions: String,                    // Uprawnienia dostępu np. rwx-r--rw-
    block_size: u32,                        // Optymalny rozmiar bloku
    block_number: u32,                      // Liczba 512 bajtowych bloków
    last_access: DateTime<Utc>,             // Data ostatniego dostępu
    last_modification: DateTime<Utc>,       // Data ostatniej modyfikacji
    last_status_changed: DateTime<Utc>,     // Data ostatniej zmiany statusu
}

impl FileInfo {
    /// Utworzenie obiektu dla pliku określonego nazwą pliku i jego katalogiem.
    pub fn new(name: &str, dir: &str) -> Result<Self> {
        let path = format!("{}/{}", dir, name);
        let file_stat = FileInfo::stat(path.as_str())?;
        
        let dt_last_access = DateTime::from_timestamp(file_stat.st_atime, 0).unwrap();
        // let dt_last_access: DateTime<Local> = dt_last_access.into();
        
        let dt_last_modification = DateTime::from_timestamp(file_stat.st_mtime, 0).unwrap();
        // let dt_last_modification: DateTime<Local> = dt_last_modification.into();
        
        let dt_last_status_changed= DateTime::from_timestamp(file_stat.st_ctime, 0).unwrap();
        // let dt_last_status_changed: DateTime<Local> = dt_last_status_changed.into();

        Ok(Self {
            name: name.into(),
            path,
            file_type: FileInfo::ftype(file_stat.st_mode),
            owner_id: file_stat.st_uid,
            owner_name: Self::user_name(file_stat.st_uid)?,
            group_id: file_stat.st_gid,
            group_name: Self::group_name(file_stat.st_gid)?,
            size: file_stat.st_size as u64,
            mode: file_stat.st_mode,
            permissions: FileInfo::file_permission(file_stat.st_mode),
            block_size: file_stat.st_blksize as u32,
            block_number: file_stat.st_blocks as u32,
            last_access: dt_last_access,
            last_modification: dt_last_modification,
            last_status_changed: dt_last_status_changed,
        })
    }

    fn validate_path(path: &str) -> Result<()> {
        match path.is_empty() {
            true => Err(Error::new(FILEINFO_ERR_CODE, FILEINFO_INVALID_PATH)),
            _ => Ok(())
        }
    }
    
    /// Utworzenie obiektu dla pliku określonego ścieżką.
    pub fn for_path(path: &str) -> Result<Self>  {
        Self::validate_path(path)?;
        
        let (path, name) = Self::split_name_and_dir(path)?;
        Self::new(&name, &path)
    }

    /// Podział ścieżki do pliku na nazwę i katalog.
    fn split_name_and_dir(path: &str) -> Result<(String, String)> {
        let slashes = path.chars()
            .enumerate()
            .filter(|(_, c)| *c == '/')
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        match slashes.last() {
            // Najpierw katalog, później nazwa.
            Some(idx) => Ok((path[..*idx].into(), path[*idx+1..].into())),
            _ => Ok((path.into(), path.into()))
        }
    }

    /// Sprawdzenie, czy plik jest katalogiem.
    pub fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    /// Sprawdzenie, czy plik jest plikiem.
    pub fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::RegularFile)
    }
    
    pub fn mode_t(&self) -> libc::mode_t {
        self.mode
    }

    /// Utworzenie reprezentacji obiektu jako text-json.
    pub fn to_json(&self) -> Result<String> {
        match serde_json::to_string(self) {
            Ok(json) => Ok(json),
            Err(e) => Err(Error::from(e))
        }
    }

    /// Utworzenie obiektu z reprezentacji text-json.
    pub fn from_json(json: &[u8]) -> Result<Self> {
        match serde_json::from_str(String::from_utf8_lossy(json).as_ref()) {
            Ok(object) => Ok(object),
            Err(e) => Err(Error::from(e))
        }
    }

    /// Zamiana typu pliku z postaci numerycznej na symboliczną (enum).
    pub fn ftype(stat: u32) -> FileType {
        match stat & libc::S_IFMT {
            libc::S_IFREG => FileType::RegularFile,
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFCHR => FileType::CharacterDevice,
            libc::S_IFBLK => FileType::BlockDevice,
            libc::S_IFIFO => FileType::PipeOrFIFO,
            libc::S_IFSOCK => FileType::Socket,
            libc::S_IFLNK => FileType::SymbolicLink,
            _ => FileType::Unknown,
        }
    }
    
    /// Zamiana praw dostępu do pliku na postać tekstową,
    fn file_permission(mode: libc::mode_t) -> String {
        let mut buffer = String::new();
        match Self::ftype(mode) {
            FileType::Directory => { buffer.push('d'); },
            FileType::SymbolicLink => { buffer.push('l'); },
            FileType::CharacterDevice => { buffer.push('c'); },
            FileType::BlockDevice => { buffer.push('b'); },
            FileType::PipeOrFIFO => { buffer.push('p'); },
            FileType::Socket => { buffer.push('s'); },
            _ => { buffer.push('-'); }
        };
        
        
        match mode & libc::S_IRUSR != 0 {
            true => buffer.push('r'),
            _ => buffer.push('-')
        };
        match mode & libc::S_IWUSR != 0 {
            true => buffer.push('w'),
            _ => buffer.push('-')
        };
        match mode & libc::S_IXUSR != 0 {
            true => match mode & libc::S_ISUID != 0 {
                true => buffer.push('s'),
                _ => buffer.push('x')
            }
            _ => match mode & libc::S_ISUID != 0 {
                true => buffer.push('S'),
                _ => buffer.push('-')
            }
        };
        match mode & libc::S_IRGRP != 0 {
            true => buffer.push('r'),
            _ => buffer.push('-')
        }
        match mode & libc::S_IWGRP != 0 {
            true => buffer.push('w'),
            _ => buffer.push('-')
        }
        match mode & libc::S_IXGRP != 0 {
            true => match mode & libc::S_ISUID != 0 {
                true => buffer.push('s'),
                _ => buffer.push('x')
            }
            _ => match mode & libc::S_ISUID != 0 {
                true => buffer.push('S'),
                _ => buffer.push('-')
            }
        }
        match mode & libc::S_IROTH != 0 {
            true => buffer.push('r'),
            _ => buffer.push('-')
        }
        match mode & libc::S_IWOTH != 0 {
            true => buffer.push('w'),
            _ => buffer.push('-')
        }
        match mode & libc::S_IXOTH != 0 {
            true => match mode & libc::S_ISUID != 0 {
                true => buffer.push('s'),
                _ => buffer.push('x')
            }
            _ => match mode & libc::S_ISUID != 0 {
                true => buffer.push('S'),
                _ => buffer.push('-')
            }
        }
        buffer.shrink_to_fit();
        buffer
    }

    /// Odczyt informacji o pliku ze wskazaną ścieżką.
    pub fn stat(path: &str) -> Result<libc::stat> {
        unsafe {
            let cstr = CString::new(path).unwrap();
            let mut status: libc::stat = std::mem::zeroed();
            match libc::stat(cstr.as_ptr(), &mut status) {
                0 => Ok(status),
                _ => Err(Error::from_errno())
            }
        }
    }

    /// Odczyt nazwy użytkownika ze wskazanym uid.
    fn user_name(uid: u32) -> Result<String> {
        unsafe {
            match libc::getpwuid(uid) {
                ptr if ptr == null_mut() => return Err(Error::from_errno()),
                passwd => {
                    let name_cstr = CStr::from_ptr((*passwd).pw_name);
                    let name_str = CStr::from_ptr(name_cstr.as_ptr()).to_str().unwrap();
                    Ok(name_str.into())
                }
            }
            
            // let passwd = libc::getpwuid(uid);
            // if passwd == null_mut() {
            //     return Err(Error::from_errno());
            // }
            // let name_cstr = CStr::from_ptr((*passwd).pw_name);
            // let name_str = CStr::from_ptr(name_cstr.as_ptr()).to_str().unwrap();
            // Ok(name_str.into())
        }
    }

    /// Odczyt nazwy grupy ze wskazanych gid.
    fn group_name(gid: u32) -> Result<String> {
        unsafe {
            let group = libc::getgrgid(gid);
            if group == null_mut() {
                return Err(Error::from_errno());
            }
            let name_cstr = CStr::from_ptr((*group).gr_name);
            let name_str = CStr::from_ptr(name_cstr.as_ptr()).to_str().unwrap();
            Ok(name_str.into())
        }
    }
    
    /// Opis obiektu w postaci czytelnego tekstu.
    pub fn as_str(&self) -> String {
        let mut bufferr = format!("[{}] (\n", self.name);
        bufferr.push_str(&format!("\t               path: {}\n", &self.path));
        bufferr.push_str(&format!("\t               type: {:?}\n", self.file_type));
        bufferr.push_str(&format!("\t               size: {:6}\n", self.size));
        bufferr.push_str(&format!("\t        permissions: {}\n", self.permissions));
        bufferr.push_str(&format!("\t      rozmiar bloku: {}\n", self.block_size));
        bufferr.push_str(&format!("\t      liczba bloków: {}\n", self.block_number));
        bufferr.push_str(&format!("\t        last access: {}\n", self.last_access.format("%Y-%m-%d %H:%M:%S")));
        bufferr.push_str(&format!("\t  last modification: {}\n", self.last_modification.format("%Y-%m-%d %H:%M:%S").to_string()));
        bufferr.push_str(&format!("\tlast status changed: {}\n", self.last_status_changed.format("%Y-%m-%d %H:%M:%S").to_string()));
        bufferr.push_str(&format!("\t             owner : {} (uid: {})\n", self.owner_name, self.owner_id));
        bufferr.push_str(&format!("\t              group: {} (gid: {})\n", self.group_name, self.group_id));
        
        bufferr.shrink_to_fit();
        bufferr
    }
} // impl FileInfo

impl Debug for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }  
}

/// Display trait dla obiektu.
impl std::fmt::Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!("{} {:6} {:6} {:6} {} {}",
                          self.permissions,
                          self.owner_name,
                          self.group_name,
                          self.size,
                          self.last_status_changed.with_timezone(&Local).format("%Y-%m-%d %H:%M"),
                          self.name);
        write!(f, "{}", str)
    }   
}
