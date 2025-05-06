// https://github.com/stamparm/cryptospecs/blob/master/symmetrical/sources/3-way.c
// http://www.users.zetnet.co.uk/hopwood/crypto/scan/cs.html#3-Way

use crate::crypto::tool::{align_to_block, bytes_to_block3, block3_to_bytes, pad_index, iv_block};

const NMBR: usize = 11;             // Liczba rund
const BLOCK_SIZE: usize = 12;       // 3xu32: 12 bajtów
pub const KEY_SIZE: usize = 12;         // Rozmiar klucza jako liczba bajtów.
const ERCON: [u32; 12] = [0x0b0b, 0x1616, 0x2c2c, 0x5858, 0xb0b0, 0x7171, 0xe2e2, 0xd5d5, 0xbbbb, 0x6767, 0xcece, 0x8d8d];
const DRCON: [u32; 12] = [0xb1b1, 0x7373, 0xe6e6, 0xdddd, 0xabab, 0x4747, 0x8e8e, 0x0d0d, 0x1a1a, 0x3434, 0x6868, 0xd0d0];

pub struct Way3 {
    k:  [u32; 3],
    ki: [u32; 3],
}

impl Way3 {
    pub fn new_with_key_block(block: (u32, u32, u32)) -> Result<Self, &'static str> {
        let mut key = vec![0u8; KEY_SIZE];
        block3_to_bytes(block, &mut key);
        Self::new(key.as_slice())
    }

    pub fn new(key: &[u8]) -> Result<Self, &'static str> {
        if key.len() != KEY_SIZE {
            return Err("invalid key size");
        }
        let mut k = [0u32; 3];
        let mut ki = [0u32; 3];
        unsafe {
            let ptr = key.as_ptr() as *const u32;
            k[0] = *ptr;
            k[1] = *(ptr.offset(1));
            k[2] = *(ptr.offset(2));
            ki[0] = *ptr;
            ki[1] = *(ptr.offset(1));
            ki[2] = *(ptr.offset(2));
        }
        Self::mu(Self::theta(&mut ki));
        Ok(Way3 { k, ki })
    }

    pub fn key_size() -> usize {
        KEY_SIZE
    }

    /****************************************************************
    *                                                               *
    *                           B L O C K                           *
    *                                                               *
    ****************************************************************/

    pub fn encrypt_block(&self, src: (u32,u32,u32)) -> (u32,u32,u32) {
        let mut a = [src.0, src.1, src.2];

        ERCON[..NMBR]
            .iter()
            .for_each(|&v| {
                a[0] ^= self.k[0] ^ (v << 16);
                a[1] ^= self.k[1];
                a[2] ^= self.k[2] ^ v;
                Self::rho(&mut a);
            });
        a[0] ^= self.k[0] ^ (ERCON[NMBR] << 16);
        a[1] ^= self.k[1];
        a[2] ^= self.k[2] ^ ERCON[NMBR];
        Self::theta(&mut a);
        
        (a[0], a[1], a[2])
    }

    pub fn decrypt_block(&self, src: (u32,u32,u32)) -> (u32,u32,u32) {
        let mut a = [src.0, src.1, src.2];
        Self::mu(&mut a);

        DRCON[..NMBR]
            .iter()
            .for_each(|&v| {
                a[0] ^= self.ki[0] ^ (v << 16);
                a[1] ^= self.ki[1];
                a[2] ^= self.ki[2] ^ v;
                Self::rho(&mut a);
            });
        a[0] ^= self.ki[0] ^ (DRCON[NMBR] << 16);
        a[1] ^= self.ki[1];
        a[2] ^= self.ki[2] ^ DRCON[NMBR];
        Self::mu(Self::theta(&mut a));

        (a[0], a[1], a[2])
    }

    /****************************************************************
    *                                                               *
    *                            E C B                              *
    *                                                               *
    ****************************************************************/

    /// Zaszyfrowanie ciągu bajtów w trybie ECB.
    /// Wielkość zaszyfrowanego ciągu ma taką samą długość ciągu szyfrowanego.
    /// Jeśli długość ciągu do zaszyfrowania nie jest wielokrotnością bloku,
    /// zostanie on uzupełniony paddingiem. 
    /// UWAGA: ten sam ciąg po zaszyfrowaniu zawsze wygląda tak samo.
    pub fn encrypt_ecb(&self, input: &[u8]) -> Vec<u8> {
        if input.is_empty() { return vec![]; }

        let plain = align_to_block(input, BLOCK_SIZE);
        let nbytes = plain.len();
        let mut cipher = vec![0u8; nbytes];

        plain.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let plain_block = bytes_to_block3(&plain[i..]);
                let cipher_block = self.encrypt_block(plain_block);
                block3_to_bytes(cipher_block, &mut cipher[i..i+BLOCK_SIZE]);                
            });
        
        cipher
    }

    /// Odszyfrowanie ciągu bajtów w trybie ECB.
    /// Długość ciągu bajtów musi być wielokrotnością długości bloków.
    pub fn decrypt_ecb(&self, cipher: &[u8]) -> Vec<u8> {
        if cipher.is_empty() { return vec![]; }

        let nbytes = cipher.len();
        let mut plain = vec![0u8; nbytes];

        cipher.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let cipher_block = bytes_to_block3(&cipher[i..]);
                let plain_block = self.decrypt_block(cipher_block);
                block3_to_bytes(plain_block, &mut plain[i..i+BLOCK_SIZE]);
            });
        
        match pad_index(&plain) {
            Some(idx) => plain[..idx].to_vec(),
            _ => plain,
        }
    }

    /****************************************************************
    *                                                               *
    *                            C B C                              *
    *                                                               *
    ****************************************************************/

    /// Zaszyfrowanie ciągu bajtów w trybie CBC.
    /// Przy szyfrowaniu używa się losowego IV.
    /// Oznacza to, że nawet jeśli wiele razy szyfrujemy
    /// ten sam tekst, po zaszyfrowaniu będzie on zawsze wyglądał inaczej.
    pub fn encrypt_cbc(&self, input: &[u8]) -> Vec<u8> {
        if input.is_empty() { return vec![]; }
        
        let plain = align_to_block(input, BLOCK_SIZE);
        let mut cipher = vec![0u8; BLOCK_SIZE + plain.len()];
        iv_block(&mut cipher[0..BLOCK_SIZE]);
        
        let mut cipher_block = bytes_to_block3(&cipher);
        plain.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let plain_block = bytes_to_block3(&plain[i..]);
                let w0 = plain_block.0 ^ cipher_block.0;
                let w1 = plain_block.1 ^ cipher_block.1;
                let w2 = plain_block.2 ^ cipher_block.2;
                cipher_block = self.encrypt_block((w0, w1, w2));
                let pos = i + BLOCK_SIZE;
                block3_to_bytes(cipher_block, &mut cipher[pos..pos+BLOCK_SIZE]);
            });
        
        cipher
    }

    /// Odszyfrowanie ciągu bajtów w trybie CBC.
    /// Długość ciągu bajtów musi być wielokrotnością długości bloku
    /// i musi zawierać co najmniej 2 bloki.
    pub fn decrypt_cbc(&self, cipher: &[u8]) -> Vec<u8> {
        let nbytes = cipher.len();
        if nbytes / BLOCK_SIZE < 2 || nbytes % BLOCK_SIZE != 0 {
            return vec![]; 
        }
        let mut plain = vec![0u8; nbytes - BLOCK_SIZE];
        
        let mut ptv_cipher_block = bytes_to_block3(cipher);
        cipher[BLOCK_SIZE..].iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let cipher_block = bytes_to_block3(&cipher[i+BLOCK_SIZE..]);
                let tmp = cipher_block;
                let plain_block = self.decrypt_block(cipher_block);
                let w0 = plain_block.0 ^ ptv_cipher_block.0;
                let w1 = plain_block.1 ^ ptv_cipher_block.1;
                let w2 = plain_block.2 ^ ptv_cipher_block.2;
                block3_to_bytes((w0, w1, w2), &mut plain[i..i+BLOCK_SIZE]);
                
                ptv_cipher_block = tmp;
            });
        
        pad_index(&plain)
            .map(|idx| plain[..idx].to_vec())
            .unwrap_or(plain)
    }

    /********************************************************************
    *                                                                   *
    *                         H E L P E R S                             *
    *                                                                   *
    ********************************************************************/

    fn mu(data: &mut [u32]) -> &mut [u32]{
        let mut a0 = data[0];
        let mut a1 = data[1];
        let mut a2 = data[2];
        let mut b0 = 0u32;
        let mut b1 = 0u32;
        let mut b2 = 0u32;

        for _i in 0..32 {
            b0 <<= 1;
            b1 <<= 1;
            b2 <<= 1;
            if (a0 & 1) != 0 { b2 |= 1; }
            if (a1 & 1) != 0 { b1 |= 1; }
            if (a2 & 1) != 0 { b0 |= 1; }
            // b0 |= a2 & 1;
            // b1 |= a1 & 1;
            // b2 |= a0 & 1;
            a0 >>= 1;
            a1 >>= 1;
            a2 >>= 1;
        }
        data[0] = b0;
        data[1] = b1;
        data[2] = b2;
        data
    }

    fn theta(data: &mut [u32]) -> &mut [u32]{
        let a0 = data[0];
        let a1 = data[1];
        let a2 = data[2];

        data[0] = a0 ^
            (a0 >> 16) ^ (a1 << 16) ^ (a1 >> 16) ^ (a2 << 16) ^
            (a1 >> 24) ^ (a2 <<  8) ^ (a2 >>  8) ^ (a0 << 24) ^
            (a2 >> 16) ^ (a0 << 16) ^ (a2 >> 24) ^ (a0 <<  8);

        data[1] = a1 ^
            (a1 >> 16) ^ (a2 << 16) ^ (a2 >> 16) ^ (a0 << 16) ^
            (a2 >> 24) ^ (a0 <<  8) ^ (a0 >>  8) ^ (a1 << 24) ^
            (a0 >> 16) ^ (a1 << 16) ^ (a0 >> 24) ^ (a1 << 8);

        data[2] = a2 ^
            (a2 >> 16) ^ (a0 << 16) ^ (a0 >> 16) ^ (a1 << 16) ^
            (a0 >> 24) ^ (a1 <<  8) ^ (a1 >>  8) ^ (a2 << 24) ^
            (a1 >> 16) ^ (a2 << 16) ^ (a1 >> 24) ^ (a2 << 8);
        data
    }

    fn gamma(data: &mut [u32]) -> &mut [u32]{
        let a0 = data[0];
        let a1 = data[1];
        let a2 = data[2];

        data[0] = a0 ^ (a1 | !a2);
        data[1] = a1 ^ (a2 | !a0);
        data[2] = a2 ^ (a0 | !a1);
        data
    }

    fn pi1(data: &mut [u32]) -> &mut [u32]{
        let a0 = data[0];
        let a2 = data[2];

        data[0] = (a0 >> 10) ^ (a0 << 22);
        data[2] = (a2 <<  1) ^ (a2 >> 31);
        data
    }

    fn pi2(data: &mut [u32]) -> &mut [u32]{
        let a0 = data[0];
        let a2 = data[2];

        data[0] = (a0 <<  1) ^ (a0 >> 31);
        data[2] = (a2 >> 10) ^ (a2 << 22);
        data
    }

    fn rho(data: &mut [u32]) -> &mut [u32]{
        Self::pi2(Self::gamma(Self::pi1(Self::theta(data))))
    }

}   // Way3

#[cfg(test)]
mod tests {
    use crate::crypto::tool::rnd_bytes;
    use super::*;

    #[test]
    fn test_block() {
        struct Test {
            key: (u32, u32, u32),
            plain: (u32, u32, u32),
            cipher: (u32, u32, u32),
        }
        let tests = [
            Test {
                key: (0, 0, 0),
                plain: (1, 1, 1),
                cipher: (0x4059c76e, 0x83ae9dc4, 0xad21ecf7),
            },
        ];

        for tt in tests {
            let w3 = Way3::new_with_key_block(tt.key);
            assert!(w3.is_ok());
            let w3 = w3.unwrap();
            let encrypted = w3.encrypt_block(tt.plain);
            assert_eq!(encrypted, tt.cipher);
        }
    }

    #[test]
    fn test_ecb() {
        let key = rnd_bytes(Way3::key_size());
        let w3 = Way3::new(&key).unwrap();
        let plain = "Piotr Pszczółkowski".as_bytes();
        let cipher = w3.encrypt_ecb(plain);
        let decrypted = w3.decrypt_ecb(&cipher);
        assert_eq!(plain, &decrypted);
    }

    #[test]
    fn test_cbc() {
        let key = rnd_bytes(Way3::key_size());
        let w3 = Way3::new(&key);
        assert!(w3.is_ok());
        let w3 = w3.unwrap();
        
        let plain = "Piotr Pszczółkowski Włodzimierz".as_bytes();
        let cipher = w3.encrypt_cbc(plain);
        let decrypted = w3.decrypt_cbc(&cipher);
        assert_eq!(plain, &decrypted);
    }
    
}