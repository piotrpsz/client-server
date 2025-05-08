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

use std::{
    io::ErrorKind, 
    env, 
    fs,
    process::Command
};
use shared::data::{
    answer::Answer,
    request::Request,
};
use shared::ufs::dir::Dir;
use shared::ufs::file::{self, File };
use shared::xerror::{ Result, Error };

static EXEC_ERR_CODE: i32 = -4;
static EXEC_INVALID_ENTRY_NAME: &str = "invalid entry name";
static EXEC_INVALID_PARAMETERS: &str = "invalid number of parameters";
static EXEC_FILE_ALREADY_EXISTS: &str = "file already exists";
static EXEC_FILE_NOT_FOUND: &str = "ile not found";
static EXEC_NO_PARAMETERS: &str = "no call parameters";
static EXEC_NO_SUCH_COMMAND: &str = "no such command";

pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> Result<Answer> {
        let params = request.params.as_slice();
        match request.command.as_str() {
            "pwd" => Self::pwd(),
            "cd" => Self::cd(params),
            "mkdir" => Self::mkdir(params),
            "ls" | "ll" => Self::ls(params),
            "la" => Self::la(params),
            "touch" => Self::touch(params),
            "rm" => Self::rm(params),
            "rmdir" => Self::rmdir(params),
            "rename" | "move" => Self::move_file(params),
            "exe" => Self::execute_cmd(params),
            _ => Err(Error::new(EXEC_ERR_CODE, EXEC_NO_SUCH_COMMAND))
        }
    }

    fn validate_params(params: &[String]) -> Result<()> {
        match params.is_empty() {
            true => Err(Error::with_error_kind(EXEC_ERR_CODE, ErrorKind::InvalidData, EXEC_NO_PARAMETERS)),
            false => Ok(()) }
    }
    
    fn execute_command(cmd: &str, args: &[String]) -> Result<Answer> {
        
        Ok(Answer::new(0, "OK", "exe"))    
    }
    
    fn execute_cmd(params: &[String]) -> Result<Answer> {
        

        let mut cmd = Command::new(params[0].as_str());
        for param in &params[1..] {
            cmd.arg(param);
        }
        let args = cmd.get_args().collect::<Vec<_>>();
        eprintln!("{:?} {:?}", cmd.get_program(), args);
        
        let output = cmd.output()?;
        let err_str = String::from_utf8_lossy(&output.stderr).to_string();
        let std_str = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(Answer::new_with_data(0, "OK", "exe", vec![std_str, err_str]))
    }
    
    /// pwd - print working directory
    fn pwd() -> Result<Answer> {
        match env::current_dir() {
            Ok(path) => {
                let mut answer = Answer::new(0, "OK", "pwd");
                answer.data.push(path.to_str().unwrap().to_string());
                Ok(answer)
            },
            Err(err) => Err(Error::from(err)),
        }
    }

    /// cd - change directory
    fn cd(params: &[String]) -> Result<Answer> {
        let mut path = match params.is_empty() {
            // Jeśli nie podano katalogu (brak parametru) to idziemy do katalogu domowego.
            true => "~".to_string(),
            false => params[0].clone()
        };
        if path.starts_with("~") {
            // Jeśli katalog zaczyna się tyldą, to ją zastępujemy
            // absolutną ścieżką do katalogu domowego.
            path = path.replace("~", dirs::home_dir().unwrap().to_str().unwrap());
        }
              
        match env::set_current_dir(path) {
            Ok(_) => {
                let mut answer = Answer::new(0, "OK", "cd");
                answer.data.push(env::current_dir()?.to_str().unwrap().to_string());
                Ok(answer)
            },
            Err(err) => Err(Error::from(err))
        }
    }
    
    /// mkdir - create directory
    fn mkdir(params: &[String]) -> Result<Answer> {
        Self::validate_params(params)?;

        let made: Result<Vec<_>> = params.iter()
            .filter(|path| Self::is_regular_name(path).is_ok())
            .map(|path| {
                match fs::create_dir_all(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(Error::from(err))
                }
            })
            .collect();

        // for param in params {
        //     match param.contains(SEPERATOR) {
        //         true => fs::create_dir_all(param)?,
        //         false => fs::create_dir(param)?
        //     }
        // }
        Ok(Answer::new_with_data(0, "OK", "mkdir", made?))
    }
    
    /// Odczyt zawartości katalogu bez plików ukrytych.
    /// Dopuszczalny jest brak parametrów (odczyt aktualnego katalogu).
    fn ls(params: &[String]) -> Result<Answer> {
        let data = Self::readdir(params, false)?;
        Ok(Answer::new_with_data(0, "OK", "ls", data))
    }
    
    /// Odczytanie zawartości katalogu wraz z plikami ukrytymi.
    /// Dopuszczalny jest brak parametrów (odczyt aktualnego katalogu). 
    fn la(params: &[String]) -> Result<Answer> {
        let data = Self::readdir(params, true)?;
        Ok(Answer::new_with_data(0, "OK", "la", data))
    }
    
    /// Odczyt zawartości katalogu, ze wskazaniem czy uwzględniać pliki ukryte.
    fn readdir(params: &[String], hidden_too: bool) -> Result<Vec<String>> {
        let dir = match params.is_empty() {
            // Jeśli nie podano katalogu (brak parametru) to czytamy aktualny katalog.
            true => ".".to_string(),
            false => params[0].clone()
        };


        let files = Dir::read(&dir, hidden_too)?;
            
        // Zamiana informacji o plikach (fileinfo) na wektor JSON.
        let data = files
            .iter()
            .map(|fi| fi.to_json().unwrap())
            .collect();

        Ok(data)
    }
    
    /// Utworzenie pustego pliku.
    fn touch(params: &[String]) -> Result<Answer> {
        Self::validate_params(&params)?;

        let made: Result<Vec<String>> = params.into_iter()
            .filter(|path| Self::is_regular_name(path).is_ok())
            .map(|path| {
                match file::touch(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();

        Ok(Answer::new_with_data(0, "OK", "touch", made?))
    }
    
    /// Usunięcie pliku
    fn rm(params: &[String]) -> Result<Answer> {
        Self::validate_params(&params)?;

        let made: Result<Vec<String>> = params.into_iter()
            .filter( |path| Self::is_regular_name(path).is_ok() )
            .map( |path| {
                match file::rm(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();

        Ok(Answer::new_with_data(0, "OK", "rm", made?))
    }
    
    fn rmdir(params: &[String]) -> Result<Answer>{
        Self::validate_params(params)?;

        match params[0].as_str() {
            "-r" => Self::rm_directories_with_content(&params[1..]),
            _ => Self::rm_directories(params)
        }
    }
    
    /// Standardowa funkcja usuwani katalogu.
    /// UWAGA: katalog musi być pusty.
    fn rm_directories(paths: &[String]) -> Result<Answer> {
        Self::validate_params(paths)?;

        let made: Result<Vec<String>> = paths.to_vec()
            .iter()
            .filter(|path| {
                Self::is_regular_name(path).is_ok()
            })
            .map(|path| {
                match Dir::rmdir(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();
        
        Ok(Answer::new_with_data(0, "OK", "rmdir", made?))
    }

/*
    fn execute_with_fn<F>(fn_executor: F, paths: &[String]) -> io::Result<Answer>
        where F: Fn(&str) -> io::Result<Answer>
    {
        let removed: Result<Vec<String>, _> = paths.to_vec()
            .iter()
            .filter(|path| {
                Self::is_regular_name(path).is_ok()
            })
            .map(|path| {
                match fn_executor(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();

        match removed {
            Ok(v) => Ok(Answer::new_with_data(0, "OK", "rmdir", v)),
            Err(err) => Err(err)
        }    
    }
 */
    
    fn rm_directories_with_content(paths: &[String]) -> Result<Answer> {
        Self::validate_params(paths)?;

        let made: Result<Vec<String>> = paths.to_vec()
            .iter()
            .filter(|path| {
                Self::is_regular_name(path).is_ok()
            })
            .map(|path| {
                match Self::rm_directory_with_content(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();

        Ok(Answer::new_with_data(0, "OK", "rmdir", made?))
    }

    /// Sprawdzenie, czy ścieżka wskazuje na normalny katalog.
    /// Normalny katalog to ten, którego nazwa nie jest '.' i '..'.
    fn is_regular_name(path: &str) -> Result<()> {
        // Wyznaczenie wszystkich pozycji znaku '/'.
        let items = path.bytes()
            .enumerate()
            .filter(|(_, c)| *c == b'/')
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let name = match items.last() {
            // Wyznaczenie ostatniej części ścieżki, czyli nazwę.
            Some(v) => path[v + 1..].to_string(),
            // Brak znaków '/' - ścieżki jest pojedyńczym wyrazem, który jest nazwą.
            _ => path.to_string()
        };

        match name.as_str() {
            "." | ".." => Err(Error::new(EXEC_ERR_CODE, EXEC_INVALID_ENTRY_NAME)),
            _ => Ok(())
        }
    }

    fn rm_directory_with_content(path: &str) -> Result<()> {
        Self::is_regular_name(path)?;

        let content = Dir::read(path, true)?;
        for fi in content {
            if fi.is_dir() {
                Self::rm_directory_with_content(fi.path.as_str())?;
            } else {
                File::new(fi.path.as_str()).rm()?;
            }
        }
        Dir::rmdir(path)
    }

    /// Przeniesienie pliku lub zmiana jego nazwy.
    fn move_file(paths: &[String]) -> Result<Answer> {
        if paths.len() != 2 {
            return Err(Error::with_error_kind(EXEC_ERR_CODE, ErrorKind::InvalidData, EXEC_INVALID_PARAMETERS));
        }

        let from = paths[0].as_str();
        let to = paths[1].as_str();
        if file::exist(to).is_ok() {
            return Err(Error::with_error_kind(EXEC_ERR_CODE, ErrorKind::AlreadyExists, EXEC_FILE_ALREADY_EXISTS));
        }
        if file::exist(from).is_err() {
            return Err(Error::with_error_kind(EXEC_ERR_CODE, ErrorKind::NotFound, EXEC_FILE_NOT_FOUND));
        }

        file::rename(from, to)?;
        Ok(Answer::new_with_data(0, "OK", "rmdir", paths.to_vec()))
    }

}


