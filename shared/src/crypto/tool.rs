#![allow(dead_code)]

use rand::RngCore;

pub(crate) fn padding(nbytes: usize) -> Vec<u8> {
    let mut padd= vec![0u8; nbytes];
    padd[0] = 128;
    padd
}

pub(crate) fn pad_index(data: &[u8]) -> Option<usize> {
    let mut i = data.len();
    
    while i > 0 {
        i -= 1;
        if data[i] != 0 {
            if data[i] == 128 {
                return Some(i);    
            }
            break;
        }
    }
    
    // for i in (data.len()-1..= 0).rev() {
    //     if data[i] != 0 {
    //         if data[i] == 128 {
    //             return Some(i);
    //         }
    //         break;
    //     }
    // }
    None
}

pub fn rnd_bytes(nbytes: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; nbytes];
    rand::rng().fill_bytes(&mut buffer);
    buffer
}

pub(crate) fn iv_block(buffer: &mut [u8]) {
    rand::rng().fill_bytes(buffer);
}


pub(crate) fn align_to_block(input: &[u8], block_size: usize) -> Vec<u8> {
    let n = input.len() % block_size;
    let padd = if n != 0 {
        padding(block_size - n)
    } else {
        vec![]
    };
    let mut output = input.to_vec();
    output.extend_from_slice(&padd);
    output
}

 pub(crate) fn bytes_to_block(data: &[u8]) -> (u32,u32) {
    unsafe {
        let ptr = data.as_ptr() as *const u32;
        let a = *ptr.offset(0);
        let b = *ptr.offset(1);
        (a,b)
    }
}

pub(crate) fn bytes_to_block3(data: &[u8]) -> (u32,u32,u32) {
    let chunks = data.chunks(4).collect::<Vec<_>>();
    let mut retv: Vec<u32> = vec![];
    for chunk in chunks.iter() {
        let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        retv.push(value);    
    }
    (retv[0], retv[1], retv[2])
/*    
    unsafe {
        let ptr = data.as_ptr() as *const u32;
        let a = *ptr.offset(0);
        let b = *ptr.offset(1);
        let c = *ptr.offset(2);
        (a,b,c)   
    }
    
 */
}

pub(crate) fn block_to_bytes(block: (u32, u32), data: &mut [u8]) {
    unsafe {
        let dst = data.as_mut_ptr();
        let src0 = &block.0 as *const u32 as *const u8;
        let src1 = &block.1 as *const u32 as *const u8;
        
        *dst.offset(0) = *src0.offset(0);
        *dst.offset(1) = *src0.offset(1);
        *dst.offset(2) = *src0.offset(2);
        *dst.offset(3) = *src0.offset(3);
        
        *dst.offset(4) = *src1.offset(0);
        *dst.offset(5) = *src1.offset(1);
        *dst.offset(6) = *src1.offset(2);
        *dst.offset(7) = *src1.offset(3);
    }
}

pub(crate) fn block3_to_bytes(block: (u32, u32, u32), data: &mut [u8]) {
    data[..4].copy_from_slice(&block.0.to_be_bytes());
    data[4..8].copy_from_slice(&block.1.to_be_bytes());
    data[8..].copy_from_slice(&block.2.to_be_bytes());
    
    /*
    unsafe {
        let dst = data.as_mut_ptr();
        let src0 = &block.0 as *const u32 as *const u8;
        let src1 = &block.1 as *const u32 as *const u8;
        let src2 = &block.2 as *const u32 as *const u8;
        
        *dst.offset(0) = *src0.offset(0);
        *dst.offset(1) = *src0.offset(1);
        *dst.offset(2) = *src0.offset(2);
        *dst.offset(3) = *src0.offset(3);
        
        *dst.offset(4) = *src1.offset(0);
        *dst.offset(5) = *src1.offset(1);
        *dst.offset(6) = *src1.offset(2);
        *dst.offset(7) = *src1.offset(3);
        
        *dst.offset(8) = *src2.offset(0);
        *dst.offset(9) = *src2.offset(1);
        *dst.offset(10) = *src2.offset(2);
        *dst.offset(11) = *src2.offset(3);
    }
   
     */
}