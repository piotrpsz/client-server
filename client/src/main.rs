// MIT License
//
// Copyright (c) 2025 Piotr Pszczółkowski
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use shared::data::{
    request::Request
};
use std::io::{ stdin, ErrorKind};
use std::net::*;
use shared::data::answer::Answer;
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;
use shared::xerror::{ Error, Result };
use ansi_term::Colour::{ Yellow, Red };

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
        eprint!("{}", Yellow.paint("cmd> "));
        // eprint!("cmd> ");
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
        _ => {
            let err = Error::from(answer);
            let err_str = format!("{:?}", err);
            eprintln!("{}", Red.paint(err_str))
        }
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
