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

use std::{ env, process::Command };
use crate::data::{answer::Answer, request::Request };
use crate::ufs::dir::Dir;
use crate::ufs::file::File;
use crate::ufs::fileinfo::FileInfo;
use crate::xerror::{Result, Error };

pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> Result<Answer> {
        match request.command.as_str() {
            // Polecenia la i ls obsługujemy osobno, bo chcemy dostać fileinfo o każdym pliku.
            "la" => Self::la(request.params.as_slice()),
            "ll" => Self::ls(request.params.as_slice()),
            // Polecenie cd obsługujemy osobno, aby obsłużyć cd bez parametrów.
            "cd" => Self::cd(request.params.as_slice()),
            // Polecenie wysłania pliku
            "put" => Self::download(request.params.as_slice()),
            // Polecenie pobrania pliku.
            "get" => Self::upload(request.params.as_slice()),
            // Własne pomysły
            "stat" => Self::stat(request.params.as_slice()),
            // Reszta standardowo.
            _ => Self::execute_command(request.command.as_str(), request.params.as_slice())
        }
    }
    
    fn execute_command(cmd: &str, args: &[String]) -> Result<Answer> {
        let mut command = Command::new(cmd);
        command.args(args);

        let output = command.output()?;
        let mut err_str = String::from_utf8_lossy(&output.stderr).to_string();
        if err_str.as_bytes().last() == Some(&b'\n')  {
            err_str.truncate(err_str.len() - 1);
        }
        let mut std_str = String::from_utf8_lossy(&output.stdout).to_string();
        if std_str.as_bytes().last() == Some(&b'\n')  {
            std_str.truncate(std_str.len() - 1);
        }
        Ok(Answer::new_with_data(0, "OK", cmd, vec![std_str, err_str]))
    }

    /// Odczyt zawartości katalogu, ze wskazaniem czy uwzględniać pliki ukryte.
    fn readdir(params: &[String], hidden_too: bool) -> Result<Vec<String>> {
        eprintln!("readdir: {:?}", params);
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
    
    /// Odczyt zawartości katalogu bez plików ukrytych.
    /// Dopuszczalny jest brak parametrów (odczyt aktualnego katalogu).
    fn ls(params: &[String]) -> Result<Answer> {
        let data = Self::readdir(params, false)?;
        Ok(Answer::new_with_data(0, "OK", "ll", data))
    }
    
    /// Odczytanie zawartości katalogu wraz z plikami ukrytymi.
    /// Dopuszczalny jest brak parametrów (odczyt aktualnego katalogu). 
    fn la(params: &[String]) -> Result<Answer> {
        let data = Self::readdir(params, true)?;
        Ok(Answer::new_with_data(0, "OK", "la", data))
    }

    fn stat(params: &[String]) -> Result<Answer> {
        let made: Result<Vec<String>> = params.iter()
            .map(|path| {
                let mut path = path.clone();
                if !path.starts_with("/") {
                    let cmd = env::current_dir()?;
                    path = cmd.to_str().unwrap().to_string() + "/" + &path;
                }
                let fi = FileInfo::for_path(path.as_str())?.to_json()?;
                Ok(fi)
            })
            .collect();
        Ok(Answer::new_with_data(0, "OK", "stat", made?))
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
    
    fn download(params: &[String]) -> Result<Answer> {
        Ok(Answer::new(0, "OK", "download"))
    }
    
    fn upload(params: &[String]) -> Result<Answer> {
        let name = params[0].clone();
        eprintln!("upload: {:?}", params);
        
        let fh = File::new(name.as_str());
        let data = fh.read_all_vec()?;
        let mut answer = Answer::new_with_data(0, "OK", "upload", params.to_vec());
        answer.binary = data;
        Ok(answer)
    }
}
