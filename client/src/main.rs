use shared::data::{
    request::Request
};

use std::io;
use std::io::{ stdin, ErrorKind};
use std::net::*;
use shared::data::answer::Answer;
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;
use shared::ufs::Error;

fn main() -> io::Result<()>{
    let addr = SocketAddr::from(([127, 0, 0, 1], 25105));
    match TcpStream::connect(addr) {
        Ok(socket) => {
            println!("Connected to server: {}", socket.peer_addr()?);
            handle_connection(socket)?;
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::ConnectionRefused => eprintln!("Server is not running."),
                _ => eprintln!("Connection error: {}", e),
            };
        },
    }
    Ok(())
}

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut conn = Connector::new(stream.try_clone()?, ConnectionSide::Client);
    conn.init()?;
    
    let mut input = String::new();
    loop {
        // Odczytaj z terminala polecenie wraz z argumentami.
        input.clear();
        eprint!("cmd> ");
        stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if input.is_empty() {
            // Pusta linia oznacza zakończenie programu..
            break;
        }
        // Wszystkie elementy oddzielone są spacjami.
        // Pierwszy element to polecenie, reszta to argumenty.
        let tokens = input.split_whitespace().collect::<Vec<&str>>();
        let command = tokens[0];
        let mut args = Vec::with_capacity(tokens.len() - 1);
        for token in tokens[1..].iter() {
            args.push(token.to_string());       
        }

        //===========================================================
        
        // Komunikacja z serwerem.
        let request = Request::new(command.into(), args);
        conn.send_request(request)?;

        let answer = conn.read_answer()?;
        display_answer(answer);
    }
    Ok(())
}

fn display_answer(answer: Answer) {
    // eprintln!("{:?}", answer);
    
    match answer.message.as_str() {
        "OK" => {
            if !answer.data.is_empty() {
                match answer.cmd.as_str() {
                    "pwd" => print_pwd_answer(answer.data),
                    "cd" => print_cd_answer(answer.data),
                    "mkdir" => print_mkdir_answer(answer.data),
                    "ls" | "la" => print_lsa_answer(answer.data),
                    "rmdir" => print_rmdir_answer(answer.data),
                    "exe" => print_exe_answer(answer.data),

                    _ => println!("{:?}", answer),
                }
            }
        },
        _ => println!("{}", Error::from(answer))
    };
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
    data.iter()
        .for_each(|item| {
            let fi = FileInfo::from_json(item.as_bytes()).unwrap();
            println!("{}", fi)
        });
}

fn print_rmdir_answer(data: Vec<String>) {
    for item in data {
        println!("{}", item);
    }
}

fn print_exe_answer(data: Vec<String>) {
    data[0].split('\n').for_each(|line| {
        if !line.is_empty() {
            println!("{}", line)
        }
    });
    data[1].split('\n').for_each(|line| {
        if !line.is_empty() {
            println!("{}", line)
        }
    });
}
