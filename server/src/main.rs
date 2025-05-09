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

extern crate core;

use std::error::Error;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::net::*;
use std::{io, thread};
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use shared::data::answer::Answer;
use shared::net::connector::{ConnectionSide, Connector};
use shared::executor::Executor;

static STOP: AtomicBool = AtomicBool::new(false);
static TASK_COUNT: AtomicU32 = AtomicU32::new(0);
static TASK_ID: AtomicU32 = AtomicU32::new(0);

fn ctrlc_handler() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = unbounded();

    ctrlc::set_handler(move || {
        STOP.store(true, std::sync::atomic::Ordering::Relaxed);
        let n = TASK_COUNT.load(std::sync::atomic::Ordering::Relaxed);
        eprintln!("Received {} tasks to stop", n);
        
        for _ in 0..n {
            sender.send(()).unwrap();       
        };
    }).expect("Error setting Ctrl-C handler");
    
    Ok(receiver)
}

fn accept(listener: &TcpListener, sender: Sender<TcpStream>) {
    match listener.accept() {
        Ok(conn) => {
            sender.send(conn.0).unwrap();
        }
        Err(e) => {
            eprintln!("Error accepting connection: {}", e);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    let ctrl_receiver = ctrlc_handler()?;
    let (accept_sender, accept_receiver) = bounded::<TcpStream>(1);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 25105));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);
    
    // Ponieważ accept jest blokujące, uruchamiamy go w dedykowanych wątku.
    let _ = thread::spawn(move || {
        loop {
            accept(&listener, accept_sender.clone());
        }
    });
    
    rayon::scope(|s| {
        TASK_COUNT.fetch_add(1, Relaxed);
        let main_task_id = TASK_ID.fetch_add(1, Relaxed);
        eprintln!("Main task started id: {}", main_task_id);
        
        loop {
            select! {
                recv(ctrl_receiver) -> _ => {
                    eprintln!("Received stop signal");
                    break;
                }
                recv(accept_receiver) -> value => {
                    match value {
                        Ok(stream) => {
                            // Jeśli nie przerwano działania programu,
                            // uruchamiamy nowy task dla obsługi połączenia
                            // z klientem.
                            if !STOP.load(Relaxed) {
                                let ctrl_channel = ctrl_receiver.clone();
                                let mut stream = stream;
                                s.spawn(move |_| {
                                    handle_client(&mut stream, ctrl_channel);
                                });
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving: {:?}", e);
                        }
                    }
                }
            }
        }
    });
    
    Ok(())
}

fn handle_client(stream: &mut TcpStream, ctrl_receiver: Receiver<()>) {
    TASK_COUNT.fetch_add(1, Relaxed);
    let task_id = TASK_ID.fetch_add(1, Relaxed);
    
    let mut conn = Connector::new(stream.try_clone().unwrap(), ConnectionSide::Server);
    let peer = conn.peer_addr();
    eprintln!("Connected client {} (tid: {})", peer, task_id);
    
    match conn.init() {
        Ok(_) => (),
        Err(why) => {
            stream.shutdown(Shutdown::Both).unwrap();
            eprintln!("Task canceled with error {} (tid:{})", why, task_id);
            TASK_COUNT.fetch_sub(1, Relaxed);
            return;
        }
    }
    
    loop {
        if ctrl_receiver.try_recv().is_ok() {
            TASK_COUNT.fetch_sub(1, Relaxed);
            eprintln!("Task canceled id: {}", task_id);
            return;
        }
        
        match one_loop(&mut conn) {  
            Ok(_) => (),
            Err(why) => {
                if why.kind() == ErrorKind::BrokenPipe || why.kind() == ErrorKind::UnexpectedEof {
                    eprintln!("Client {} disconnected (tid: {})", peer, task_id);
                } else {
                    eprintln!("** Error executing: {}", why);
                }
                TASK_COUNT.fetch_sub(1, Relaxed);
                return;
            }
        }
    }
}

/// Jedna sekwencja zapytanie-odpowiedź.
/// Dla na błąd wykonania polecenia nie jest błędem.
/// Dla nas błędem są problemy komunikacji z klientem.
fn one_loop(conn: &mut Connector) -> io::Result<()> {
    let request = conn.read_request()?;
    eprintln!("-- received request: {}", request.to_pretty_json()?);
    
    match Executor::execute(request) {
        Ok(answer) => {
            conn.send_answer(answer)?;
            eprintln!("-- sent answer: OK");
            Ok(())
        },
        Err(err) => {
            let answer = Answer::from(err);
            match conn.send_answer(answer) {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        }
    }
}
