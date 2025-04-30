use shared::data::{
    answer::Answer,
    request::Request
};

use std::io;
use std::io::{ stdin, ErrorKind};
use std::net::*;
use shared::data::message::Message;
use shared::net::connector::{ConnectionSide, Connector};

fn main() -> io::Result<()>{
    // use shared::crypto::tool::rnd_bytes;
    // use shared::crypto::blowfish;
    // let key = rnd_bytes(128);
    // eprintln!("{:02x?}", key);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 25105));
    match TcpStream::connect(addr) {
        Ok(socket) => {
            println!("Connected to server: {}", socket.peer_addr()?);
            handle_connection(socket)?;
        }
        Err(e) => {
            if e.kind() == ErrorKind::ConnectionRefused {
                eprintln!("Server is not running");
            } else {
                eprintln!("Connection error: {}", e);
            }
        },
    }
    Ok(())
}

fn handle_connection(mut socket: TcpStream) -> io::Result<()> {
    let mut conn = Connector::new(socket.try_clone()?, ConnectionSide::Client);
    let mut input = String::new();
    loop {
        input.clear();
        eprint!("Give command: ");
        stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if input.is_empty() {
            break;
        }
        
        let tokens = input.split_whitespace().collect::<Vec<&str>>();
        let command = tokens[0];
        let mut args = Vec::with_capacity(tokens.len() - 1);
        for token in tokens[1..].iter() {
            args.push(token.to_string());       
        }

        
        let request = Request::new(command.into(), args);
        conn.send_request(request)?;

        let answer = conn.read_answer()?;
        println!("-- received: {}", answer.to_pretty_json()?);
    }
    Ok(())
}