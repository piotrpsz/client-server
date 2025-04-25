use std::{
    io,
    env
};
use shared::data::{
    answer::Answer,
    request::Request
};

pub struct Executor;

impl Executor {
    pub fn execute(request: Request) -> Result<Answer, io::Error> {
        if request.command == "pwd" {
            return Executor::pwd();
        }
        
        Err(
            io::Error::new(
                io::ErrorKind::Other,
                "Command not found"
            )
        )
        
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
}
