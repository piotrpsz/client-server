use shared::data::{
    request::Request
};

use std::io;
use std::io::{ stdin, ErrorKind};
use std::net::*;
use shared::data::answer::Answer;
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;

fn main() -> io::Result<()>{
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

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut conn = Connector::new(stream.try_clone()?, ConnectionSide::Client);
    conn.init()?;
    
    let mut input = String::new();
    loop {
        input.clear();
        eprint!("cmd> ");
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

        //===========================================================
        
        let request = Request::new(command.into(), args);
        conn.send_request(request)?;

        let answer = conn.read_answer()?;
        display_answer(answer);
        // println!("-- received: {}", answer.to_pretty_json()?);
    }
    Ok(())
}

fn display_answer(answer: Answer) {
    match answer.message.as_str() {
        "OK" => {
            match answer.cmd.as_str() {
                "pwd" => print_pwd_answer(answer.data),
                "cd" => print_cd_answer(answer.data),
                "mkdir" => print_mkdir_answer(answer.data),
                "ls" | "la" => print_lsa_answer(answer.data),
                "rmdir" => print_rmdir_answer(answer.data),
                _ => eprintln!("{:?}", answer),
            }
        },
        _ => print_error(answer)
    }
}

fn print_pwd_answer(data: Vec<String>) { 
    println!("{}", data[0]);
}

fn print_cd_answer(data: Vec<String>) {
    println!("{}", data[0]);
}

fn print_mkdir_answer(data: Vec<String>) {
    for item in data {
        println!("{}", item);
    }
}

fn print_lsa_answer(data: Vec<String>) {
    for item in data {
        let fi = FileInfo::from_json(item.as_bytes()).unwrap();
        println!("{}", fi);
    }
}

fn print_rmdir_answer(data: Vec<String>) {
    println!("{}", data[0]);
}

fn print_error(answer: Answer) {
    if !answer.data.is_empty() {
        let text = format!("{}: {} (kind: {})", answer.message, answer.data[0], answer.data[1]);
        println!("{:?}", text);
    }
}