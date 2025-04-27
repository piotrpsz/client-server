mod executor;

use std::error::Error;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::net::*;
use std::thread;
use crossbeam_channel::{unbounded, bounded, Sender, Receiver, select};
use shared::data::{
        message::Message,
        request::Request,
        answer::Answer
};
use shared::crypto::blowfish::Blowfish;

use crate::executor::Executor;

static STOP: AtomicBool = AtomicBool::new(false);
static TASK_COUNT: AtomicU32 = AtomicU32::new(0);
static TASK_ID: AtomicU32 = AtomicU32::new(0);

struct TcpInfo {
    addr: SocketAddr,
    conn: TcpStream,
}

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

fn accept(listener: &TcpListener, sender: Sender<TcpInfo>) {
    match listener.accept() {
        Ok((conn, addr)) => {
            sender.send(TcpInfo{addr, conn}).unwrap();
        }
        Err(e) => {
            eprintln!("Error accepting connection: {}", e);
        }
    }
}

fn main() {
    eprintln!("Hello, world!");
    let key = "TESTKEY".as_bytes();
    let bf = Blowfish::new(key).unwrap();
    let a = 1u32;
    let b = 2u32;
    let expected_a =  0xdf333fd2u32;
    let expected_b =  0x30a71bb4u32;
    let result = bf.encrypt(a, b);
    // assert_eq!(result.0, expected_a);
    // assert_eq!(result.1, expected_b);
    eprintln!("{:x}", expected_a );
    eprintln!("{:x}", expected_b );
    if result.0 == expected_a && result.1 == expected_b {
        eprintln!("Encryption test passed");
    }
    
    let result1 = bf.decrypt(expected_a, expected_b);
    if result1.0 == a && result1.1 == b {
        eprintln!("Decryption test passed");
    }
    
}
/*
fn main() -> Result<(), Box<dyn Error>>{
    let ctrl_receiver = ctrlc_handler()?;
    let (accept_sender, accept_receiver) = bounded::<TcpInfo>(1);
    
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
                        Ok(tcp) => {
                            // Jeśli nie przerwano działania programu,
                            // uruchamiamy nowy task dla obsługi połączenia
                            // z klientem.
                            if !STOP.load(Relaxed) {
                                let ctrl_channel = ctrl_receiver.clone();
                                let mut tcp_info = tcp;
                                s.spawn(move |_| {
                                    handle_client(&mut tcp_info, ctrl_channel);
                                });
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving: {:?}", e);
                        }
                    }
                }
            };
        }
    });
    
    Ok(())
}
*/
fn handle_client(tcp: &mut TcpInfo, ctrl_receiver: Receiver<()>) {
    TASK_COUNT.fetch_add(1, Relaxed);
    let task_id = TASK_ID.fetch_add(1, Relaxed);
    eprintln!("Connected client {} (tid: {})", tcp.addr, task_id);
    
    loop {
        if ctrl_receiver.try_recv().is_ok() {
            TASK_COUNT.fetch_sub(1, Relaxed);
            eprintln!("Task canceled id: {}", task_id);
            return;
        }
        
        match Message::read(&mut tcp.conn) {
            Ok(buffer) => {
                let request = Request::from_json(&buffer).unwrap();
                println!("-- received request: {}", request.to_pretty_json().unwrap());
                
                
                match Executor::execute(request) {
                    Ok(answer) => {
                        Message::write(&mut tcp.conn, answer.to_json().unwrap().as_bytes()).unwrap();
                        println!("-- sent answer: {}", answer.to_pretty_json().unwrap());
                    },
                    Err(e) => {
                        let mut answer = Answer::new(1, "ERROR".into());
                        answer.data.push(e.to_string());
                        let answer = answer.to_json().unwrap();
                        Message::write(&mut tcp.conn, answer.as_bytes()).unwrap();
                        eprintln!("** Error executing request: {}", e);
                        
                    }
                }
                
                
                // let answer = Answer::new(0, "OK".into()).to_json().unwrap();
                // Message::write(&mut tcp.conn, answer.as_bytes()).unwrap();
                // println!("-- sent answer: {:?}", answer);
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => 
                        eprintln!("Client {} disconnected (tid: {})", tcp.addr, task_id),
                    _ => 
                        eprintln!("** Error reading data: {}", e)
                }
                TASK_COUNT.fetch_sub(1, Relaxed);
                return;
            }
        }
    }
}
