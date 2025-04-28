const NMBR: isize = 11;             // Liczba rund
const BLOCK_SIZE: isize = 12;       // 3xu32: 12 bajtów
const KEY_SIZE: usize = 12;         // Rozmiar klucza jako liczba bajtów.
const ERCON: [u32; 12] = [0x0b0b, 0x1616, 0x2c2c, 0x5858, 0xb0b0, 0x7171, 0xe2e2, 0xd5d5, 0xbbbb, 0x6767, 0xcece, 0x8d8d];
const DRCON: [u32; 12] = [0xb1b1, 0x7373, 0xe6e6, 0xdddd, 0xabab, 0x4747, 0x8e8e, 0x0d0d, 0x1a1a, 0x3434, 0x6868, 0xd0d0];

pub struct Way3 {
    k:  [u32; 3],
    ki: [u32; 3],
}

impl Way3 {
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
            (a0 >> 16) ^ (a1 << 16) ^
            (a1 >> 16) ^ (a2 << 16) ^
            (a1 >> 24) ^ (a2 <<  8) ^
            (a2 >>  8) ^ (a0 << 24) ^
            (a2 >> 16) ^ (a0 << 16) ^
            (a2 >> 24) ^ (a0 <<  8);

        data[1] = a1 ^
            (a1 >> 16) ^ (a2 << 16) ^
            (a2 >> 16) ^ (a0 << 16) ^
            (a2 >> 24) ^ (a0 <<  8) ^
            (a0 >>  8) ^ (a1 << 24) ^
            (a0 >> 16) ^ (a1 << 16) ^
            (a0 >> 24) ^ (a1 << 8);

        data[2] = a2 ^
            (a2 >> 16) ^ (a0 << 16) ^
            (a0 >> 16) ^ (a1 << 16) ^
            (a0 >> 24) ^ (a1 <<  8) ^
            (a1 >>  8) ^ (a2 << 24) ^
            (a1 >> 16) ^ (a2 << 16) ^
            (a1 >> 24) ^ (a2 << 8);
        data
    }
    
    fn gamma(data: &mut [u32]) -> &mut [u32]{
        let a0 = data[0];
        let a1 = data[1];
        let a2 = data[2];
    
        data[0] = !a0 ^ (!a1 & a2);
        data[1] = !a1 ^ (!a2 & a0);
        data[2] = !a2 ^ (!a0 & a1);
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