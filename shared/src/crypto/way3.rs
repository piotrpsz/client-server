// http://www.users.zetnet.co.uk/hopwood/crypto/scan/cs.html#3-Way

const NMBR: usize = 11;             // Liczba rund
const BLOCK_SIZE: isize = 12;       // 3xu32: 12 bajtów
const KEY_SIZE: usize = 12;         // Rozmiar klucza jako liczba bajtów.
const ERCON: [u32; 12] = [0x0b0b, 0x1616, 0x2c2c, 0x5858, 0xb0b0, 0x7171, 0xe2e2, 0xd5d5, 0xbbbb, 0x6767, 0xcece, 0x8d8d];
const DRCON: [u32; 12] = [0xb1b1, 0x7373, 0xe6e6, 0xdddd, 0xabab, 0x4747, 0x8e8e, 0x0d0d, 0x1a1a, 0x3434, 0x6868, 0xd0d0];

pub struct Way3 {
    k:  [u32; 3],
    ki: [u32; 3],
}

impl Way3 {
    pub fn new_with_key_block(key: &[u32; 3]) -> Result<Self, &'static str> {
        let ptr = key.as_ptr() as *const u8;
        let mut k = [0u8; KEY_SIZE];
        eprintln!("1. {:x?}", key);
        unsafe {
            for i in 0..KEY_SIZE {
                k[i] = *ptr.add(i);
            }
        }
        Self::new(k.as_slice())
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
        eprintln!("2. {:x?}", k);
        Self::mu(Self::theta(&mut ki));
        Ok(Way3 { k, ki })
    }
    
    pub fn encrypt_block(&self, src: &[u32], dst: &mut [u32]) {
        let mut a = [0u32; 3];
        a[0] = src[0];
        a[1] = src[1];
        a[2] = src[2];
        
        for i in 0..NMBR {
            a[0] ^= self.k[0] ^ (ERCON[i] << 16);
            a[1] ^= self.k[1];
            a[2] ^= self.k[2] ^ ERCON[i];
            Self::rho(&mut a);
        }
        a[0] ^= self.k[0] ^ (ERCON[NMBR] << 16);
        a[1] ^= self.k[1];
        a[2] ^= self.k[2] ^ ERCON[NMBR];
        
        Self::theta(&mut a);
        dst[0] = a[0];
        dst[1] = a[1];
        dst[2] = a[2];
    }
    
    pub fn decrypt_block(&self, src: &mut [u32], dst: &mut [u32]) {
        let mut a = [0u32; 3];
        a[0] = src[0];
        a[1] = src[1];
        a[2] = src[2];
        Self::mu(&mut a);
        
        for i in 0..NMBR {
            a[0] ^= self.ki[0] ^ (DRCON[i] << 16);
            a[1] ^= self.ki[1];
            a[2] ^= self.ki[2] ^ DRCON[i];
            Self::rho(&mut a);
        }
        a[0] ^= self.ki[0] ^ (DRCON[NMBR] << 16);
        a[1] ^= self.ki[1];
        a[2] ^= self.ki[2] ^ DRCON[NMBR];
        
        Self::mu(Self::theta(&mut a));
        dst[0] = a[0];
        dst[1] = a[1];
        dst[2] = a[2];
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
            b0 |= a2 & 1;
            b1 |= a1 & 1;
            b2 |= a0 & 1;
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
        data[2] = (a2 << 1) ^ (a2 >> 31);
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
    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_block() {
        struct Test {
            key: [u32; 3],
            plain: [u32; 3],
            cipher: [u32; 3],
        }
        let tests = [
            Test {
                key: [0, 0, 0],
                plain: [1, 1, 1],
                cipher: [0x4059c76e, 0x83ae9dc4, 0xad21ecf7],
            },
            Test {
                key: [4, 5, 6],
                plain: [1, 2, 3],
                cipher: [0xd2f05b5e, 0xd6144138, 0xcab920cd]
            }
        ];

        let mut buffer = [0u32; 3];
        for tt in tests {
            let w3 = Way3::new_with_key_block(&tt.key);
            assert!(w3.is_ok());
            let w3 = w3.unwrap();
            w3.encrypt_block(tt.plain.as_slice(), &mut buffer);
            eprintln!("{:x?}", buffer);
            assert_eq!(buffer, tt.cipher);
        }
    }
}