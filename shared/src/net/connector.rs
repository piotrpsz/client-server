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
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use crate::crypto::{blowfish, blowfish::Blowfish, gost, gost::Gost, way3, way3::Way3};
use crate::crypto::tool::rnd_bytes;
use crate::data::{message::Message, request::Request, answer::Answer };

const BF_KEY: [u8; blowfish::MAX_KEY_SIZE] = [
    0xbe, 0x2f, 0xe0, 0xa8, 0xd9, 0xc9, 0xec, 0x31, 0x06, 0x67,
    0x7a, 0x1b, 0xe6, 0x93, 0xdc, 0x72, 0xaf, 0xa1, 0xfa, 0x68,
    0xc4, 0x59, 0x02, 0x05, 0xd3, 0xf8, 0xf1, 0xd4, 0x6e, 0x38,
    0x84, 0x12, 0x68, 0x12, 0x6e, 0x7a, 0x4a, 0xb7, 0xd9, 0x21,
    0x93, 0x23, 0xe9, 0x90, 0xe3, 0xf2, 0xf2, 0xec, 0x6b, 0x36,
    0x66, 0xa9, 0x51, 0xa9, 0xb6, 0x71];

const CLIENT_ID: [u8; 128] = [
    0x28, 0xb6, 0x01, 0xb3, 0xc4, 0x9c, 0x16, 0xf5, 0xa4, 0x53,
    0x16, 0xd0, 0x00, 0xc8, 0xab, 0x1d, 0xb5, 0x70, 0x5f, 0xe1,
    0x92, 0x45, 0x0c, 0x6c, 0x39, 0xdb, 0x88, 0x69, 0x84, 0xd6,
    0x18, 0x00, 0x93, 0xc6, 0x7d, 0x95, 0xab, 0xc3, 0xf0, 0xb8,
    0x15, 0x7f, 0x2f, 0x4e, 0x64, 0x48, 0xe0, 0xa1, 0x75, 0xe9,
    0x2f, 0x20, 0xc1, 0x8f, 0x42, 0x93, 0x24, 0x71, 0x29, 0xe1,
    0x7b, 0x36, 0xc0, 0x02, 0x49, 0x99, 0x98, 0x0e, 0x08, 0xab,
    0xd7, 0x82, 0x70, 0x55, 0x27, 0x5f, 0x73, 0xf1, 0x24, 0x29,
    0xbd, 0xa0, 0x1e, 0x14, 0xe0, 0x99, 0xc8, 0x70, 0xd5, 0x56,
    0x55, 0x86, 0xfd, 0x44, 0x2b, 0x83, 0xbf, 0xd1, 0x03, 0x46,
    0x08, 0x28, 0x3f, 0x95, 0xa8, 0x8a, 0x34, 0xe7, 0xfd, 0x52,
    0xba, 0x6b, 0x74, 0xd8, 0x13, 0xdc, 0x16, 0x85, 0xd5, 0x4e,
    0x6e, 0x08, 0xf1, 0xa2, 0x4f, 0x94, 0x88, 0xa3];

pub enum ConnectionSide {
    Server,
    Client
}

pub struct Connector {
    conn: TcpStream,
    side: ConnectionSide,
    blowfish: Blowfish,
    gost: Option<Gost>,
    way3: Option<Way3>,
    prv_request: Option<Request>,
    prv_answer: Option<Answer>,
        
}

impl Connector {
    pub fn new(conn: TcpStream, side: ConnectionSide ) -> Self {
        Connector {
            conn,
            side,
            blowfish: Blowfish::new(BF_KEY.as_slice()).unwrap(),
            gost: None,
            way3: None,
            prv_request: None,
            prv_answer: None,
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        match self.side {
            ConnectionSide::Server => self.init_server(),
            ConnectionSide::Client => self.init_client()
        }
    } // fn init
    
    fn init_server(&mut self) -> io::Result<()> {
        self.read_client_id()?;
        self.send_keys()
    } // fn init_sever
    
    fn init_client(&mut self) -> io::Result<()> {
        self.send_client_id()?;
        self.read_keys()
    } // fn init_client

    fn send_client_id(&mut self) -> io::Result<()> {
        let data = self.blowfish.encrypt_cbc(&CLIENT_ID);
        Message::write(&mut self.conn, &data)
    } // fn send_client_id
    
    fn read_client_id(&mut self) -> io::Result<()> {
        let client_id = Message::read(&mut self.conn)?;
        let client_id = self.blowfish.decrypt_cbc(&client_id);
        if client_id != CLIENT_ID {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid client-id.")); 
        }
        Ok(())
    } // fn read_client_id

    /// Serwer wysyła klucze szyfrowania dla GOST i 3-Way.
    /// Te klucze są losowo generowane dla jednej, tej konkretnej, sesji.
    fn send_keys(&mut self) -> io::Result<()> {
        let gost_key = rnd_bytes(gost::KEY_SIZE);
        let way3_key = rnd_bytes(way3::KEY_SIZE);
        let mut keys = vec![];
        keys.extend_from_slice(gost_key.as_slice());
        keys.extend_from_slice(way3_key.as_slice());
        // Klucze szyfrujemy Blowfishem i wysyłamy do klienta.
        let data = self.blowfish.encrypt_cbc(keys.as_slice());
        Message::write(&mut self.conn, data.as_slice())?;
        // Po udanym wysłaniu kluczy używamy ich do
        // utworzenia silników szyfrowania po stronie serwera.
        self.gost = Some(Gost::new(gost_key.as_slice()).unwrap());
        self.way3 = Some(Way3::new(way3_key.as_slice()).unwrap());
        Ok(())
    } // fn send_keys

    /// Klient odczytuje klucze szyfrowania dla GOST i 3-Way.
    /// Serwer je losowo wygenerował na użytek tej sesji.
    fn read_keys(&mut self) -> io::Result<()> {
        // Odczyt kluczy i ich odszyfrowanie Blowfishem.
        let keys = Message::read(&mut self.conn)?;
        let keys = self.blowfish.decrypt_cbc(keys.as_slice());
        if keys.len() != gost::KEY_SIZE + way3::KEY_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid keys length."));
        }
        // Po udanych odczycie kluczy używamy ich do
        // utworzenia silników szyfrowania po stronie klienta.
        let gost_key = keys[..gost::KEY_SIZE].to_vec();
        let way3_key = keys[gost::KEY_SIZE..].to_vec();
        self.gost = Some(Gost::new(gost_key.as_slice()).unwrap());
        self.way3 = Some(Way3::new(way3_key.as_slice()).unwrap());
        Ok(())
    } // fn read_keys

    pub fn peer_addr(&self) -> String {
        self.conn.peer_addr().unwrap().to_string()
    }
    pub fn local_addr(&self) -> String {
        self.conn.local_addr().unwrap().to_string()
    }
    
    //------- Serwer ------------------------------------------------
    
    /// Odczytanie żądania.
    /// Żądanie zapamiętujemy?
    pub fn read_request(&mut self) -> io::Result<Request> {
        let data = Message::read(&mut self.conn)?;
        let request = self.blowfish.decrypt_cbc(&data);
        let request = Request::from_json(&request)?;
        if self.prv_answer.is_some() && request.id() != (self.prv_answer.as_ref().unwrap().id() + 1) {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid request id."));
        }
        self.prv_request = Some(request.clone());
        Ok(request)
    } // fn read_request
    
    /// Wysłanie zapytania.
    /// Przed wysłaniem zapytania musimy uzupełnić ID.
    /// Numer ID pobieramy z poprzedniego zapytania. 
    /// Wysłane zapytanie zapamiętujemy.
    pub fn send_answer(&mut self, mut answer: Answer) -> io::Result<()> {
        let id = match self.prv_request {
            Some(ref request) => request.id(),
            None => 0 };
        answer.set_id(id + 1);
        let data = self.blowfish.encrypt_cbc(answer.to_json()?.as_bytes());
        Message::write(&mut self.conn, data.as_slice())?;
        self.prv_answer = Some(answer);
        Ok(())
    } // fn send_answer

    //------- Klient ------------------------------------------------
    
    pub fn send_request(&mut self, mut request: Request) -> io::Result<()> {
        let id = match self.prv_answer {
            Some(ref request) => request.id(),
            None => 0 };
        request.set_id(id + 1);
        let data = self.blowfish.encrypt_cbc(request.to_json()?.as_bytes());
        Message::write(&mut self.conn, data.as_slice())?;
        // Jeśli zapis się zakończył sukcesem, zapamiętujemy to żądanie. 
        self.prv_request = Some(request);
        Ok(())
    } // fn send_request
    
    pub fn read_answer(&mut self) -> io::Result<Answer> {
        let data = Message::read(&mut self.conn)?;
        let answer = self.blowfish.decrypt_cbc(&data);
        let answer = Answer::from_json(&answer)?;
        if self.prv_answer.is_some() && answer.id() != (self.prv_request.as_ref().unwrap().id() + 1) {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid answer id."));
        }
        // Jeśli wszystko poszło dobrze, zapamiętujemy odpowiedź.
        self.prv_answer = Some(answer.clone());
        Ok(answer)
    } // fn read_answer
     
    /// Szyfrowanie: encrypt-decrypt-encrypt (Blowfish-GOST-Way3).
    fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let data = self.blowfish.encrypt_cbc(data);
        let data = self.gost.as_ref().unwrap().decrypt_cbc(data.as_slice());
        self.way3.as_ref().unwrap().encrypt_cbc(data.as_slice())
    } // fn encrypt
    
    /// Odszyfrowanie: decrypt-encrypt-decrypt (Way3-GOST-Blowfish). 
    fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let data = self.way3.as_ref().unwrap().decrypt_cbc(data);
        let data = self.gost.as_ref().unwrap().encrypt_cbc(data.as_slice());
        self.blowfish.decrypt_cbc(data.as_slice())
    } // fn decrypt
    
} // Connector
