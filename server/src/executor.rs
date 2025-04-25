use std::{
    io,
    io::Error,
    path::PathBuf,
    env
};

use shared::data::{
    answer::Answer,
    request::Request
};
use dirs;

pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> Result<Answer, io::Error> {
        match request.command.as_str() {
            "pwd" => Self::pwd(),
            "cd" => Self::cd(request.params),
            _ => Err(Error::new(io::ErrorKind::Other, "Command not found"))
        }
    }
    
    fn pwd() -> Result<Answer, io::Error> {
        match env::current_dir() {
            Ok(path) => {
                let mut answer = Answer::new(0, "OK".into());
                answer.data.push(path.to_str().unwrap().to_string());
                Ok(answer)
            },
            Err(e) => Err(e),
        }
    }

    fn cd(params: Vec<String>) -> Result<Answer, io::Error> {
        let path = match params.is_empty() {
            true => dirs::home_dir().unwrap(),
            false => PathBuf::from(params[0].clone())
        };
        
        match env::set_current_dir(path) {
                Ok(_) => {
                    let mut answer = Answer::new(0, "OK".into());
                    answer.data.push(env::current_dir()?.to_str().unwrap().to_string());
                    Ok(answer)
                },
                Err(e) => Err(e)
            }
        }
    }

