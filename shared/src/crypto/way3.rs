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
        
        todo!()
    }
}