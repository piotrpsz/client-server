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
    request::Request,
    answer::Answer
};
use std::io::ErrorKind;
use std::net::*;
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;
use shared::xerror::{ Error, Result };
use ansi_term::Colour::{ Yellow, Red };
use rustyline::{ DefaultEditor, error::ReadlineError };


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
    
    // let mut input = String::new();
    let mut edt = DefaultEditor::new().unwrap();
    if edt.load_history("cmd_history.txt").is_err() {
        println!("No previous history.");
    }
    
    let prompt = Yellow.paint("cmd> ").to_string();
    loop { 
        let line = edt.readline(prompt.as_str());
        match line {  
            Ok(line) => {
                edt.add_history_entry(line.as_str()).expect("can't add to history");
                serve_line(&mut conn, line)?;
            },
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                println!("Bye!");
                break;
            },
            _ => ()
        }
        edt.save_history("cmd_history.txt").unwrap();
    }
    Ok(())
}

fn serve_line(conn: &mut Connector, line: String) -> Result<()>{
    let line = line.trim().to_string();
    if !line.is_empty() {
        let tokens = line.split_whitespace().collect::<Vec<&str>>();
        let command = tokens[0].to_string();
        let args = tokens[1..]
            .iter()
            .map(|item| item.to_string())
            .collect();
        
        let request = Request::new(command, args);
        conn.send_request(request)?;
        let answer = conn.read_answer()?;
        display_answer(answer);
    }
    Ok(())
}

fn display_answer(answer: Answer) {
    match answer.message.as_str() {
        "OK" => {
            if !answer.data.is_empty() {
                match answer.cmd.as_str() {
                    "ll" | "la" => print_file_info(answer.data),
                    _ => print_common(answer.data),
                }
            }
            
            // print_common(answer.data);
            // if !answer.data.is_empty() {
            //     match answer.cmd.as_str() {
            //         "pwd" => print_common(answer.data),
            //         "cd" => print_common(answer.data),
            //         "mkdir" => print_common(answer.data),
            //         "ls" | "la" => print_file_info(answer.data),
            //         "rmdir" => print_common(answer.data),
            //         "exe" => print_exe_answer(answer.data),
            // 
            //         _ => println!("{:?}", answer),
            //     }
            // }
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
         .for_each(|item| {
            if !item.is_empty() {
                println!("{}", item)
            }
        });
}

fn print_file_info(data: Vec<String>) {
    data.iter()
        .for_each(|item| {
            let fi = FileInfo::from_json(item.as_bytes()).unwrap();
            println!("{}", fi);
        });
}

