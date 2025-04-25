use std::io;
use std::io::{Read, Write};
use std::net::*;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

pub struct Message;

impl Message {
    pub fn write(stream: &mut TcpStream, buffer: &[u8]) -> io::Result<()> {
        // Write the length of the buffer as u32
        stream.write_u32::<NetworkEndian>(buffer.len() as u32)?;
        // Write the buffer
        stream.write_all(buffer)?;
        Ok(())
    }
    
    pub fn read(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
        let message_length = stream.read_u32::<NetworkEndian>()? as usize;
        let mut buffer = vec![0; message_length];
        stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}
