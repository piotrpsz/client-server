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

mod side;

use shared::data::{request::Request, answer::Answer};
use std::{net::*, io::ErrorKind};
use shared::net::connector::{ConnectionSide, Connector};
use shared::ufs::fileinfo::FileInfo;
use shared::xerror::{Error, Result};
use ansi_term::Colour::*;
use rustyline::{
    DefaultEditor,
    error::ReadlineError,
    KeyEvent,
    Event,
    ConditionalEventHandler,
    RepeatCount,
    EventContext,
    Cmd, Cmd::AcceptLine,
    EventHandler};
use shared::executor::Executor;
use shared::ufs::file::File;
use crate::side::Side;

static mut REMOTE_HOST: bool = true;

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

struct SwitchContext;
impl ConditionalEventHandler for SwitchContext {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, _ctx: &EventContext) -> Option<Cmd> {
        unsafe { REMOTE_HOST = !REMOTE_HOST; }
        Some(AcceptLine)
    }
}

/// Obsługa połączenie z serwerem.
/// Odczytujemy polecenia z linii poleceń,
/// wysyłamy do serwera i wyświetlamy wynik.
fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut conn = Connector::new(stream.try_clone()?, ConnectionSide::Client);
    conn.init()?;
    serve_line_remote(&mut conn, "cd".to_string(), false)?;
    let mut side = Side::new()?;
    
    let mut edt = DefaultEditor::new().unwrap();
    edt.bind_sequence(
        Event::KeySeq(vec![KeyEvent::ctrl('Q')]),
        EventHandler::Conditional(Box::new(SwitchContext)));

    if edt.load_history("cmd_history.txt").is_err() {
        println!("No previous history.");
    }
    
    let bye = Green.paint("Bye").to_string();

    loop {
        if unsafe{ REMOTE_HOST } {
            side.set_remote(&mut conn)?;
        } else {
            side.set_local()?;
        }
        
        let line = edt.readline(side.prompt().as_str());
        match line {  
            Ok(line) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    edt.add_history_entry(line.as_str()).expect("can't add to history");
                    if side.remote {
                        serve_line_remote(&mut conn, line, true)?;
                    } else {
                        serve_line(line, true)?;
                    }
                }
            },
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                println!("{}", bye);
                break;
            },
            _ => ()
        }
        edt.save_history("cmd_history.txt").unwrap();
    }
    Ok(())
}

/// Wykonanie polecenia lokalnie.
fn serve_line(line: String, display: bool) -> Result<Answer>{
    let tokens = line.split_whitespace().collect::<Vec<&str>>();
    let command = tokens[0].to_string();
    let args = tokens[1..]
        .iter()
        .map(|item| item.to_string())
        .collect();

    let request = Request::new(command, args);
    let answer = Executor::execute(request)?;
    if display {
        display_answer(&answer);       
    }
    Ok(answer)
}

/// Wykonanie polecenia zdalnie
fn serve_line_remote(conn: &mut Connector, line: String, display: bool) -> Result<Answer>{
    let tokens = line.split_whitespace().collect::<Vec<&str>>();
    let command = tokens[0].to_string();
    let args = tokens[1..]
        .iter()
        .map(|item| item.to_string())
        .collect();

    // "get", "send"
    let request = Request::new(command, args);
    conn.send_request(request)?;
    let answer = conn.read_answer()?;
    if answer.cmd == "upload" {
        let mut fh = File::new(answer.data[0].as_str());
        fh.create()?;
        fh.write(answer.binary.as_slice())?;
        fh.close()?;
    }
    if display {
        display_answer(&answer);
    }
    Ok(answer)
}

fn display_answer(answer: &Answer) {
    match answer.message.as_str() {
        "OK" => {
            if !answer.data.is_empty() {
                match answer.cmd.as_str() {
                    "ll" | "la" => print_file_info(&answer.data),
                    "stat" =>print_stat(&answer.data),
                    _ => print_common(&answer.data),
                }
            }
        },
        _ => {
            let err = Error::from(answer.clone());
            let err_str = format!("{:?}", err);
            eprintln!("{}", Red.paint(err_str))
        }
    };
}

fn print_common(data: &[String]) {
    data.iter()
         .for_each(|item| {
            if !item.is_empty() {
                println!("{}", item)
            }
        });
}

fn print_file_info(data: &[String]) {
    data.iter()
        .for_each(|item| {
            let fi = FileInfo::from_json(item.as_bytes()).unwrap();
            println!("{}", fi);
        });
}

fn print_stat(data: &[String]) {
    data.iter()
        .for_each(|item| {
            match FileInfo::from_json(item.as_bytes()) {
                Ok(fi) => println!("{:?}", fi),
                Err(err) => println!("{:?}", err)
            }
        });
}
