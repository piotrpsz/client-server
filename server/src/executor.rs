use std::{io, env, fs};
use shared::data::{
    answer::Answer,
    request::Request,
};

use shared::ufs::dir::Dir;
use shared::ufs::{file, Error};
use shared::ufs::file::File;


static SEPERATOR: &str = "/";
pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> io::Result<Answer> {
        match request.command.as_str() {
            "pwd" => Self::pwd(),
            "cd" => Self::cd(request.params),
            "mkdir" => Self::mkdir(request.params),
            "ls" | "ll" => Self::ls(request.params),
            "la" => Self::la(request.params),
            "touch" => Self::touch(request.params),
            "rm" => Self::rm(request.params),
            "rmdir" => Self::rmdir(request.params),
            "rename" | "move" => Self::move_file(request.params),
            "exe" => Self::execute_cmd(request.params),
            _ => Err(io::Error::new(io::ErrorKind::Other, "command not found"))
        }
    }

    fn execute_cmd(params: Vec<String>) -> io::Result<Answer> {
        use std::process::Command;

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
        if params.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "No call parameters"));
        }
        match params[0].as_str() {
            "-r" => Self::rm_directories_with_content(&params[1..]),
            _ => Self::rm_directories(params.as_slice())
        }
    }

    /// Standardowa funkcja usuwani katalogu.
    /// UWAGA: katalog musi być pusty.
    fn rm_directories(paths: &[String]) -> io::Result<Answer> {
        let removed: Result<Vec<String>, _> = paths.to_vec()
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
        
        match removed {
            Ok(v) => Ok(Answer::new_with_data(0, "OK", "rmdir", v)),
            Err(err) => Err(err.into())
        }
    }

    fn execute_with_fn<F>(executor: F, paths: &[String]) -> io::Result<Answer>
        where F: Fn(&str) -> io::Result<Answer>
    {
        let removed: Result<Vec<String>, _> = paths.to_vec()
            .iter()
            .filter(|path| {
                Self::is_regular_name(path).is_ok()
            })
            .map(|path| {
                match executor(path) {
                    Ok(_) => Ok(path.clone()),
                    Err(err) => Err(err)
                }
            })
            .collect();

        match removed {
            Ok(v) => Ok(Answer::new_with_data(0, "OK", "rmdir", v)),
            Err(err) => Err(err)
        }    }
    
    fn rm_directories_with_content(paths: &[String]) -> io::Result<Answer> {
        let removed: Result<Vec<String>, _> = paths.to_vec()
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

        match removed {
            Ok(v) => Ok(Answer::new_with_data(0, "OK", "rmdir", v)),
            Err(err) => Err(err)
        }
        
        // let mut removed = Vec::with_capacity(paths.len());
        // for path in paths {
        //     Self::is_regular_name(path)?;
        //     Self::rm_directory_with_content(path.as_str())?;
        //     removed.push(path.clone());
        // }
        // Ok(Answer::new_with_data(0, "OK", "rmdir", removed))
    }
    
    fn rm_directory_with_content(path: &str) -> io::Result<()> {
        if Self::is_regular_name(path).is_ok() {
            let content = Dir::read(path, true)?;
            for fi in content {
                if fi.is_dir() {
                    Self::rm_directory_with_content(fi.path.as_str())?;
                } else {
                    File::new(fi.path.as_str()).rm()?;
                }
            }
            Dir::rmdir(path)?;
        }
        Ok(())
    }
    
    fn move_file(paths: Vec<String>) -> io::Result<Answer> {
        if paths.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid number of parameters"));
        }
        let from = paths[0].as_str();
        let to = paths[1].as_str();
        
        if file::exist(to).is_ok() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "File already exists"));
        }
        if file::exist(from).is_err() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }
        file::rename(from, to)?;
        Ok(Answer::new_with_data(0, "OK", "rmdir", paths))
    }
    

    
    /// Sprawdzenie, czy ścieżka wskazuje na normalny katalog.
    /// Normalny katalog to ten, którego nazwa nie jest '.' i '..'.
    fn is_regular_name(path: &str) -> io::Result<()> {
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
            "." | ".."  => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid entry name")),
            _ => Ok(())
        }
    }
}


