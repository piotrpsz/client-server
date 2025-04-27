use std::io;
use std::io::{Read, Write};
use std::net::*;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use crate::crypto::blowfish;

pub struct Message;

impl Message {
    pub fn write(conn: &mut TcpStream, buffer: &[u8]) -> io::Result<()> {
        conn.write_u32::<NetworkEndian>(buffer.len() as u32)?;
        conn.write_all(buffer)?;
        Ok(())
    }
    
    pub fn read(conn: &mut TcpStream) -> io::Result<Vec<u8>> {
        let message_length = conn.read_u32::<NetworkEndian>()? as usize;
        let mut buffer = vec![0; message_length];
        conn.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}
