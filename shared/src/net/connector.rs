// Moduł odpowiedzialny za komunikację poprzez socket.
// 1. Zarówno serwer, jak i klient znają klucz do Blowfisha.
// 2. Klient po nawiązaniu połączenia przesyła swój identyfikator (128 bajtów) zaszyfrowany Blowfishem,
// 3. Jeśli identyfikator jest OK, serwer odsyła hasła dla GOST i 3Wwy (zaszyfrowane Blowfishem).
// Od tej pory cała komunikacja szyfrowana jest BG3 (szyfr Blowfish - deszyfr GOST - szyfr 3Way3)

use std::io;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use crate::crypto::{blowfish, blowfish::Blowfish, gost, gost::Gost, way3, way3::Way3};
use crate::crypto::tool::rnd_bytes;
use crate::data::message::Message;

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
}

impl Connector {
    pub fn new(conn: TcpStream, side: ConnectionSide ) -> Self {
        Connector {
            conn,
            side,
            blowfish: Blowfish::new(BF_KEY.as_slice()).unwrap(),
            gost: None,
            way3: None,
        }
    }

    pub fn init(&mut self) -> Result<(),Error> {
        match self.side {
            ConnectionSide::Server => self.init_server(),
            ConnectionSide::Client => self.init_client()
        }
    } // fn init
    
    fn init_server(&mut self) -> Result<(),Error> {
        self.read_client_id()?;
        self.send_keys()
    } // fn init_sever
    
    fn init_client(&mut self) -> Result<(),Error> {
        self.send_client_id()?;
        self.read_keys()
    } // fn init_client

    fn send_client_id(&mut self) -> Result<(),Error> {
        let data = self.blowfish.encrypt_cbc(&CLIENT_ID);
        Message::write(&mut self.conn, &data)
    } // fn send_client_id
    
    fn read_client_id(&mut self) -> Result<(),Error> {
        let client_id = Message::read(&mut self.conn)?;
        if client_id.len() != CLIENT_ID.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid client-id length.")); 
        }
        Ok(())
    } // fn read_client_id

    fn send_keys(&mut self) -> Result<(),Error> {
        let gost_key = rnd_bytes(gost::KEY_SIZE);
        let way3_key = rnd_bytes(way3::KEY_SIZE);
        let mut keys = vec![];
        keys.extend_from_slice(&gost_key);
        keys.extend_from_slice(&way3_key);
        Message::write(&mut self.conn, &keys)?;

        self.gost = Some(Gost::new(&gost_key).unwrap());
        self.way3 = Some(Way3::new(&way3_key).unwrap());
        Ok(())
    } // fn send_keys

    fn read_keys(&mut self) -> Result<(), Error> {
        let keys = Message::read(&mut self.conn)?;
        let gost_key = keys[..gost::KEY_SIZE].to_vec();
        let way3_key = keys[gost::KEY_SIZE..].to_vec();
        if gost_key.len() != gost::KEY_SIZE || way3_key.len() != way3::KEY_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid keys length."));
        }
        self.gost = Some(Gost::new(&gost_key).unwrap());
        self.way3 = Some(Way3::new(&way3_key).unwrap());
        Ok(())
    } // fn read_keys
}
