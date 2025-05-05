
use std::{io, env, fs};
use shared::data::{
    answer::Answer,
    request::Request,
};

use shared::ufs::dir::Dir;
use shared::ufs::file::File;

static SEPERATOR: &str = "/";
pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> io::Result<Answer> {
        match request.command.as_str() {
            "pwd" => Self::pwd(),
            "cd" => Self::cd(request.params),
            "mkdir" => Self::mkdir(request.params),
            "ls" => Self::ls(request.params),
            "la" => Self::la(request.params),
            "touch" => Self::touch(request.params),
            "rm" => Self::rm(request.params),
            "rmdir" => Self::rmdir(request.params),
            _ => Err(io::Error::new(io::ErrorKind::Other, "Command not found"))
        }
    }
    
    /// pwd - print working directory
    fn pwd() -> io::Result<Answer> {
        match env::current_dir() {
            Ok(path) => {
                let mut answer = Answer::new(0, "OK", "pwd");
                answer.data.push(path.to_str().unwrap().to_string());
                Ok(answer)
            },
            Err(why) => Err(why),
        }
    }

    /// cd - change directory
    fn cd(params: Vec<String>) -> io::Result<Answer> {
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
            Err(why) => Err(why)
        }
    }
    
    /// mkdir - create directory
    fn mkdir(params: Vec<String>) -> io::Result<Answer> {
        if params.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "No call parameters"));
        }

        for param in &params {
            match param.contains(SEPERATOR) {
                true => fs::create_dir_all(param)?,
                false => fs::create_dir(param)?
            }
        }
        Ok(Answer::new_with_data(0, "OK", "mkdir", params))
    }
    
    /// ls - list directory
    fn ls(params: Vec<String>) -> io::Result<Answer> {
        let data = Self::readdir(params, false)?;
        Ok(Answer::new_with_data(0, "OK", "ls", data))
    }
    
    /// la - list directory with hidden files
    fn la(params: Vec<String>) -> io::Result<Answer> {
        let data = Self::readdir(params, true)?;
        Ok(Answer::new_with_data(0, "OK", "la", data))
    }
    
    /// Odczyt zawartości katalogu, ze wskazaniem czy uwzględniać pliki ukryte.
    fn readdir(params: Vec<String>, hidden_too: bool) -> io::Result<Vec<String>> {
        let dir = match params.is_empty() {
            // Jeśli nie podano katalogu (brak parametru) to czytamy aktualny katalog.
            true => ".".to_string(),
            false => params[0].clone()
        };
            
        let files = Dir::read(&dir, hidden_too)?;
            
        // Zamiana informacji o plikach na wektor JSON.
        let mut data = vec![];
        for fi in files {
            data.push(fi.to_json()?);
        }
        Ok(data)
    }
    
    /// Utworzenie pustego pliku.
    fn touch(params: Vec<String>) -> io::Result<Answer> {
        for item in &params {
            let retv = File::new(item).touch();
            if let Some(err) = retv.err() {
                return Err(err.into())
            }
        }
        Ok(Answer::new_with_data(0, "OK", "touch", params))
    }
    
    /// Usunięcie pliku
    fn rm(params: Vec<String>) -> io::Result<Answer> {
        for item in &params {
            let retv = File::new(item).rm();
            if let Some(err) = retv.err() {
                return Err(err.into())
            }       
        }
        Ok(Answer::new_with_data(0, "OK", "rm", params))
    }
    
    fn rmdir(params: Vec<String>) -> io::Result<Answer>{
        unimplemented!()
    }
}


