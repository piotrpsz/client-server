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

use std::io;
use std::io::{Read, Write};
use std::net::*;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

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
