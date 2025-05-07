use shared::data::{
    request::Request
};
use std::io::{ stdin, ErrorKind};
use std::net::*;
use shared::data::answer::Answer;
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;
use shared::xerror::{ Error, Result };

fn main() -> Result<()>{
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

fn handle_connection(stream: TcpStream) -> Result<()> {
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
                    "pwd" => print_common(answer.data),
                    "cd" => print_common(answer.data),
                    "mkdir" => print_common(answer.data),
                    "ls" | "la" => print_file_info(answer.data),
                    "rmdir" => print_common(answer.data),
                    "exe" => print_exe_answer(answer.data),

                    _ => println!("{:?}", answer),
                }
            }
        },
        _ => println!("{:?}", Error::from(answer))
    };
}

fn print_common(data: Vec<String>) {
    data.iter()
        .for_each(|item| println!("{}", item));
}

fn print_file_info(data: Vec<String>) {
    data.iter()
        .for_each(|item| {
            let fi = FileInfo::from_json(item.as_ref()).unwrap();
            println!("{}", fi);
        });
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
