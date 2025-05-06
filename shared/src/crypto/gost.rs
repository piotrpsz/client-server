use crate::crypto::tool::{align_to_block, block_to_bytes, bytes_to_block, iv_block, pad_index};

const BLOCK_SIZE: usize = 8;
// 8 bytes = 2 u32 = 54 bit
pub const KEY_SIZE: usize = 32;
// 32 bytes = 8 u32 = 256 bit
const K8: [u8; 16] = [14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7];
const K7: [u8; 16] = [15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10];
const K6: [u8; 16] = [10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8];
const K5: [u8; 16] = [7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15];
const K4: [u8; 16] = [2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9];
const K3: [u8; 16] = [12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11];
const K2: [u8; 16] = [4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1];
const K1: [u8; 16] = [13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7];

pub struct Gost {
    k0: u32,
    k1: u32,
    k2: u32,
    k3: u32,
    k4: u32,
    k5: u32,
    k6: u32,
    k7: u32,
    k87: [u8; 256],
    k65: [u8; 256],
    k43: [u8; 256],
    k21: [u8; 256],
}

impl Gost {
    pub fn new_with_text_key<T: AsRef<str>>(key: T) -> Result<Self, &'static str> {
        Self::new(key.as_ref().as_bytes())
    }
    
    pub fn new(key: &[u8]) -> Result<Self, &'static str> {
        if key.len() != KEY_SIZE {
            return Err("invalid key size");
        }
        
        let mut k = [0u32; 8];
        let mut k87 = [0u8; 256];
        let mut k65 = [0u8; 256];
        let mut k43 = [0u8; 256];
        let mut k21 = [0u8; 256];
        
        for i in 0usize..256 {
            let idx1 = i >> 4;
            let idx2 = i & 0x0F;
            k87[i] = (K8[idx1] << 4) ^ K7[idx2];
            k65[i] = (K6[idx1] << 4) ^ K5[idx2];
            k43[i] = (K4[idx1] << 4) ^ K3[idx2];
            k21[i] = (K2[idx1] << 4) ^ K1[idx2];
        }

        k.iter_mut()
            .enumerate()
            .for_each(|(i, k)| {
                let mut idx = (i * 4) + 3;
                let mut v = 0u32;
                v = (v << 8) + (key[idx] as u32);
                idx -= 1;
                v = (v << 8) + (key[idx] as u32);
                idx -= 1;
                v = (v << 8) + (key[idx] as u32);
                idx -= 1;
                v = (v << 8) + (key[idx] as u32);
                *k = v;

            });
        
        let ptr = &k as *const u32;
        let k0 = unsafe { *ptr.offset(0) };
        let k1 = unsafe { *ptr.offset(1) };
        let k2 = unsafe { *ptr.offset(2) };
        let k3 = unsafe { *ptr.offset(3) };
        let k4 = unsafe { *ptr.offset(4) };
        let k5 = unsafe { *ptr.offset(5) };
        let k6 = unsafe { *ptr.offset(6) };
        let k7 = unsafe { *ptr.offset(7) };

        Ok(Gost { k0, k1, k2, k3, k4, k5, k6, k7, k87, k65, k43, k21 })
    }

    fn f(&self, x: u32) -> u32 {
        let i0 = (x.wrapping_shr(24) & 0xff) as usize;
        let i1 = (x.wrapping_shr(16) & 0xff) as usize;
        let i2 = (x.wrapping_shr(8) & 0xff) as usize;
        let i3 = (x & 0xff) as usize;

        let w0 = unsafe { *self.k87.get_unchecked(i0) } as u32;
        let w1 = unsafe { *self.k65.get_unchecked(i1) } as u32;
        let w2 = unsafe { *self.k43.get_unchecked(i2) } as u32;
        let w3 = unsafe { *self.k21.get_unchecked(i3) } as u32;

        let x = w0.wrapping_shl(24)
            | w1.wrapping_shl(16)
            | w2.wrapping_shl(8)
            | w3;

        x.wrapping_shl(11) | x.wrapping_shr(32 - 11)
    }
    
    /****************************************************************
    *                                                               *
    *                           B L O C K                           *
    *                                                               *
    ****************************************************************/
    
    pub fn encrypt_block(&self, x: (u32,u32)) -> (u32,u32) {
        self.encrypt(x.0, x.1)
    }
    fn encrypt(&self, mut xl: u32, mut xr: u32) -> (u32, u32) {
        xr ^= self.f(xl.wrapping_add(self.k0));
        xl ^= self.f(xr.wrapping_add(self.k1));
        xr ^= self.f(xl.wrapping_add(self.k2));
        xl ^= self.f(xr.wrapping_add(self.k3));
        xr ^= self.f(xl.wrapping_add(self.k4));
        xl ^= self.f(xr.wrapping_add(self.k5));
        xr ^= self.f(xl.wrapping_add(self.k6));
        xl ^= self.f(xr.wrapping_add(self.k7));

        xr ^= self.f(xl.wrapping_add(self.k0));
        xl ^= self.f(xr.wrapping_add(self.k1));
        xr ^= self.f(xl.wrapping_add(self.k2));
        xl ^= self.f(xr.wrapping_add(self.k3));
        xr ^= self.f(xl.wrapping_add(self.k4));
        xl ^= self.f(xr.wrapping_add(self.k5));
        xr ^= self.f(xl.wrapping_add(self.k6));
        xl ^= self.f(xr.wrapping_add(self.k7));

        xr ^= self.f(xl.wrapping_add(self.k0));
        xl ^= self.f(xr.wrapping_add(self.k1));
        xr ^= self.f(xl.wrapping_add(self.k2));
        xl ^= self.f(xr.wrapping_add(self.k3));
        xr ^= self.f(xl.wrapping_add(self.k4));
        xl ^= self.f(xr.wrapping_add(self.k5));
        xr ^= self.f(xl.wrapping_add(self.k6));
        xl ^= self.f(xr.wrapping_add(self.k7));

        xr ^= self.f(xl.wrapping_add(self.k7));
        xl ^= self.f(xr.wrapping_add(self.k6));
        xr ^= self.f(xl.wrapping_add(self.k5));
        xl ^= self.f(xr.wrapping_add(self.k4));
        xr ^= self.f(xl.wrapping_add(self.k3));
        xl ^= self.f(xr.wrapping_add(self.k2));
        xr ^= self.f(xl.wrapping_add(self.k1));
        xl ^= self.f(xr.wrapping_add(self.k0));

        (xr, xl)
    }

    fn decrypt_block(&self, x: (u32, u32)) -> (u32, u32) {
        self.decrypt(x.0, x.1)
    }
    pub fn decrypt(&self, mut xl: u32, mut xr: u32) -> (u32, u32) {
        xr ^= self.f(xl.wrapping_add(self.k0));
        xl ^= self.f(xr.wrapping_add(self.k1));
        xr ^= self.f(xl.wrapping_add(self.k2));
        xl ^= self.f(xr.wrapping_add(self.k3));
        xr ^= self.f(xl.wrapping_add(self.k4));
        xl ^= self.f(xr.wrapping_add(self.k5));
        xr ^= self.f(xl.wrapping_add(self.k6));
        xl ^= self.f(xr.wrapping_add(self.k7));

        xr ^= self.f(xl.wrapping_add(self.k7));
        xl ^= self.f(xr.wrapping_add(self.k6));
        xr ^= self.f(xl.wrapping_add(self.k5));
        xl ^= self.f(xr.wrapping_add(self.k4));
        xr ^= self.f(xl.wrapping_add(self.k3));
        xl ^= self.f(xr.wrapping_add(self.k2));
        xr ^= self.f(xl.wrapping_add(self.k1));
        xl ^= self.f(xr.wrapping_add(self.k0));

        xr ^= self.f(xl.wrapping_add(self.k7));
        xl ^= self.f(xr.wrapping_add(self.k6));
        xr ^= self.f(xl.wrapping_add(self.k5));
        xl ^= self.f(xr.wrapping_add(self.k4));
        xr ^= self.f(xl.wrapping_add(self.k3));
        xl ^= self.f(xr.wrapping_add(self.k2));
        xr ^= self.f(xl.wrapping_add(self.k1));
        xl ^= self.f(xr.wrapping_add(self.k0));

        xr ^= self.f(xl.wrapping_add(self.k7));
        xl ^= self.f(xr.wrapping_add(self.k6));
        xr ^= self.f(xl.wrapping_add(self.k5));
        xl ^= self.f(xr.wrapping_add(self.k4));
        xr ^= self.f(xl.wrapping_add(self.k3));
        xl ^= self.f(xr.wrapping_add(self.k2));
        xr ^= self.f(xl.wrapping_add(self.k1));
        xl ^= self.f(xr.wrapping_add(self.k0));

        (xr, xl)
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
        let mut cipher = vec![0u8; plain.len()];
        
        plain.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let plain_block  = bytes_to_block(&plain[i..]);
                let cipher_block = self.encrypt_block(plain_block);
                block_to_bytes(cipher_block, &mut cipher[i..i+BLOCK_SIZE]);
            });
        
        cipher
    }

    /// Odszyfrowanie ciągu bajtów w trybie ECB.
    /// Długość ciągu bajtów musi być wielokrotnością długości bloków.
    pub fn decrypt_ecb(&self, cipher: &[u8]) -> Vec<u8> {
        if cipher.is_empty() || cipher.len() % BLOCK_SIZE != 0 {
            return vec![]; 
        }
        let mut plain = vec![0u8; cipher.len()];

        cipher.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let cipher_block = bytes_to_block(&cipher[i..]);
                let plain_block = self.decrypt_block(cipher_block);
                block_to_bytes(plain_block, &mut plain[i..i+BLOCK_SIZE]);
            });

        match pad_index(&plain) {
            Some(idx) => plain[..idx].to_vec(),
            None => plain,
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

        let mut cipher_block = bytes_to_block(&cipher);
        plain.iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let plain_block = bytes_to_block(&plain[i..]);
                let w0 = plain_block.0 ^ cipher_block.0;
                let w1 = plain_block.1 ^ cipher_block.1;
                cipher_block = self.encrypt(w0, w1);
                block_to_bytes(cipher_block, &mut cipher[(i + BLOCK_SIZE)..(i+BLOCK_SIZE+8)]);            
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

        let mut prv_cipher_block = bytes_to_block(cipher);
        cipher[BLOCK_SIZE..].iter()
            .enumerate()
            .step_by(BLOCK_SIZE)
            .for_each(|(i, _)| {
                let cipher_block = bytes_to_block(&cipher[i+BLOCK_SIZE..]);
                let tmp = cipher_block;
                let plain_block = self.decrypt_block(cipher_block);
                let w0 = plain_block.0 ^ prv_cipher_block.0;
                let w1 = plain_block.1 ^ prv_cipher_block.1;
                block_to_bytes((w0, w1), &mut plain[i..i+BLOCK_SIZE]);
                prv_cipher_block = tmp;
            });

        pad_index(&plain)
            .map(|idx| plain[..idx].to_vec())
            .unwrap_or(plain)
    }
} // Gost

#[cfg(test)]
mod tests {
    use crate::crypto::tool::rnd_bytes;
    use super::*;

    #[test]
    fn test_block() {
        let key = vec![0u8, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0];
        let gt = Gost::new(&key);
        assert!(gt.is_ok());
        let gt = gt.unwrap();

        let plain = [
            (0u32, 0u32),
            (1u32, 0u32),
            (0u32, 1u32),
            (0xffffffffu32, 0xffffffffu32)
        ];
        let expected = [
            (0x37ef7123u32, 0x361b7184u32),
            (0x1159d751u32, 0xff9b91d2u32),
            (0xc79c4ef4u32, 0x27ac9149u32),
            (0xf9709623u32, 0x56ad8d77u32)
        ];

        for i in 0..plain.len() {            
            let encrypted = gt.encrypt_block(plain[i]);
            assert_eq!(expected[i], encrypted);
            
            let decrypted = gt.decrypt_block(encrypted);
            assert_eq!(plain[i], decrypted);
        }
    }

    #[test]
    fn test_ecb() {
        let key = [0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0];
        let gt = Gost::new(&key);
        assert!(gt.is_ok());
        let gt = gt.unwrap();

        let plain = "Artur, Błażej, Jolanta i Piotr Pszczółkowscy".as_bytes();
        let cipher = gt.encrypt_ecb(plain);
        let result = gt.decrypt_ecb(&cipher);
        assert_eq!(result, plain);
    }

    #[test]
    fn test_cbc() {
        let key = rnd_bytes(32);
        let gt = Gost::new(&key);
        assert!(gt.is_ok());
        let gt = gt.unwrap();

        let plain = [
            "".as_bytes(),
            "Piotr".as_bytes(),
            "Piotr Włodzimierz Pszczółkowski".as_bytes(),
            "Yamato & Musashi".as_bytes(),
        ];
        
        for text in plain.iter() {
            let cipher = gt.encrypt_cbc(text);
            let result = gt.decrypt_cbc(&cipher);
            assert_eq!(result, text.to_vec());
        }
    }
}

