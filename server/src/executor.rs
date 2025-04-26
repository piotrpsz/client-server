use std::{io, env, fs};
use shared::data::{
    answer::Answer,
    request::Request,
    metadata::EntryMetadata,
};

static SEPERATOR: &str = "/";
pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> Result<Answer, io::Error> {
        match request.command.as_str() {
            "pwd" => Self::pwd(),
            "cd" => Self::cd(request.params),
            "mkdir" => Self::mkdir(request.params),
            "readdir" | "ls" => Self::readdir(request.params),
            _ => Err(io::Error::new(io::ErrorKind::Other, "Command not found"))
        }
    }
    
    fn pwd() -> Result<Answer, io::Error> {
        match env::current_dir() {
            Ok(path) => {
                let mut answer = Answer::new(0, "OK".into());
                answer.data.push(path.to_str().unwrap().to_string());
                Ok(answer)
            },
            Err(why) => Err(why),
        }
    }

    fn cd(params: Vec<String>) -> Result<Answer, io::Error> {
        let mut path = match params.is_empty() {
            true => "~".to_string(),
            false => params[0].clone()
        };
        if path.starts_with("~") {
            path = path.replace("~", dirs::home_dir().unwrap().to_str().unwrap());
        }
              
        match env::set_current_dir(path) {
                Ok(_) => {
                    let mut answer = Answer::new(0, "OK".into());
                    answer.data.push(env::current_dir()?.to_str().unwrap().to_string());
                    Ok(answer)
                },
                Err(why) => Err(why)
            }
        }
    
        fn mkdir(params: Vec<String>) -> Result<Answer, io::Error> {
            if params.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "No call parameters"));
            }

            for param in params {
                match param.contains(SEPERATOR) {
                    true => fs::create_dir_all(param)?,
                    false => fs::create_dir(param)?
                }
            }
            Ok(Answer::new(0, "OK".into()))       
        }
    
        fn readdir(params: Vec<String>) -> Result<Answer, io::Error> {
            let dir = match params.is_empty() {
                true => ".".to_string(),
                false => params[0].clone()
            };
            
            let mut data = Vec::new();
            for entry in fs::read_dir(dir)? {
                data.push(EntryMetadata::from(entry?).to_json()?);
            }
            
            Ok(Answer::new_with_data(0, "OK".into(), data))
        }
    }

