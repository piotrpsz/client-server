mod executor;

use std::error::Error;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::net::*;
use std::{io, thread};
use crossbeam_channel::{unbounded, bounded, Sender, Receiver, select};
use shared::data::{
        message::Message,
        request::Request,
        answer::Answer
};
use shared::crypto::blowfish::Blowfish;
use shared::net::connector::{ConnectionSide, Connector};
use crate::executor::Executor;

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

/*
fn main() {
    // use shared::crypto::crypto;
    // let data = vec![1u8, 12, 3, 14, 5, 6, 17, 8, 9, 10];
    // let block = crypto::bytes_to_block(&data);
    // let mut buffer = vec![0u8; 16];
    // crypto::block_to_bytes(block, &mut buffer);
    // eprintln!("{:?}", buffer);
    
    
    let key = "TESTKEY".as_bytes();
    let bf = Blowfish::new(key).unwrap();
    
    let text = "Piotr";
    let cipher = bf.encrypt_cbc(text.as_bytes());
    let plain = bf.decrypt_cbc(cipher.as_slice());
    assert_eq!(plain, text.as_bytes());
    eprintln!("{:?}", String::from_utf8_lossy(plain.as_slice()));
    
    // let a = 1u32;
    // let b = 2u32;
    // let expected_a =  0xdf333fd2u32;
    // let expected_b =  0x30a71bb4u32;
    // let result = bf.encrypt(a, b);
    // // assert_eq!(result.0, expected_a);
    // // assert_eq!(result.1, expected_b);
    // eprintln!("{:x}", expected_a );
    // eprintln!("{:x}", expected_b );
    // if result.0 == expected_a && result.1 == expected_b {
    //     eprintln!("Encryption test passed");
    // }
    // 
    // let result1 = bf.decrypt(expected_a, expected_b);
    // if result1.0 == a && result1.1 == b {
    //     eprintln!("Decryption test passed");
    // }

}
*/

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

fn one_loop(conn: &mut Connector) -> io::Result<()> {
    let request = conn.read_request()?;
    eprintln!("-- received request: {}", request.to_pretty_json()?);
    let answer = Executor::execute(request)?;
    eprintln!("-- sent answer: {}", answer.to_pretty_json()?);
    conn.send_answer(answer)
}
