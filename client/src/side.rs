use crate::{serve_line, serve_line_remote};
use shared::xerror::Result;
use ansi_term::Colour::*;
use shared::net::connector::Connector;

#[derive(Debug, Default)]
pub struct Side {
    pub remote: bool,
    pub local_user_name: String,
    pub local_host_name: String,
    pub remote_user_name: String,
    pub remote_host_name: String,
}

impl Side {
    pub fn new() -> Result<Self> {
        let mut side = Side::default();
        side.set_local()?;
        Ok(side)   
    }
    
    pub fn set_remote(&mut self, conn: &mut Connector) -> Result<()> {
        let host_answer = serve_line_remote(conn, "uname -n".into(), false)?;
        let user_answer = serve_line_remote(conn, "whoami".into(), false)?;
        self.remote = true;
        self. remote_host_name = host_answer.data[0].clone();
        self.remote_user_name = user_answer.data[0].clone();
        Ok(())    
    }
    
    pub fn set_local(&mut self) -> Result<()>{
        let host_answer = serve_line("uname -n".into(), false)?;
        let user_answer = serve_line("whoami".into(), false)?;
        self.remote = false;
        self.local_host_name = host_answer.data[0].clone();
        self.local_user_name = user_answer.data[0].clone();
        Ok(())
    }
    
    pub fn prompt(&self) -> String {
        match self.remote {
            true => {
                let text = format!("{}@{}> ", self.remote_user_name, self.remote_host_name);
                Yellow.paint(text).to_string()
            },
            _ => {
                let text = format!("{}@{}> ", self.local_user_name, self.local_host_name);
                Cyan.paint(text).to_string()
            },
        }
    }
}
