use shared::data::{
    answer::Answer,
    request::Request
};

use std::io;
use std::io::{ Read, Write, ErrorKind};
use std::net::*;
use byteorder::{ NetworkEndian, ReadBytesExt, WriteBytesExt };

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

fn handle_connection(mut socket: TcpStream) -> io::Result<()> {
    {
        let request = Request::new("pwd".into(), vec![]);
        send_message(&mut socket, request.to_json().unwrap().as_bytes())?;
        println!("-- sent: {:?}", request);
        
        let answer = receive_message(&mut socket)?;
        let answer = Answer::from_json(&answer);
        println!("-- received: {:?}", answer);
    }

    {
        let request = Request::new("cd".into(), vec![]);
        send_message(&mut socket, request.to_json().unwrap().as_bytes())?;
        println!("-- sent: {:?}", request);

        let answer = receive_message(&mut socket)?;
        let answer = Answer::from_json(&answer);
        println!("-- received: {:?}", answer);
    }

    {
        let request = Request::new("cd".into(), vec!["/home/piotr/Projects".into()]);
        send_message(&mut socket, request.to_json().unwrap().as_bytes())?;
        println!("-- sent: {:?}", request);

        let answer = receive_message(&mut socket)?;
        let answer = Answer::from_json(&answer);
        println!("-- received: {:?}", answer);
    }
    Ok(())
}

fn send_message(socket: &mut TcpStream, message: &[u8]) -> io::Result<()> {
    socket.write_u32::<NetworkEndian>(message.len() as u32)?;
    socket.write_all(message)?;
    Ok(())
}

fn receive_message(socket: &mut TcpStream) -> io::Result<Vec<u8>> {
    let size = socket.read_u32::<NetworkEndian>()?;
    let mut message = vec![0; size as usize];
    socket.read_exact(&mut message)?;
    Ok(message)
}
