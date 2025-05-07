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
    None
}

pub fn rnd_bytes(nbytes: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; nbytes];
    rand::rng().fill_bytes(&mut buffer);
    buffer
}

/// Wypełnienie wskazanego bufora losowymi bajtami.
/// Te losowe bajty to tzw. wektor IV.
/// Wypełniony zostanie cały bufor (standardowo BLOCK_SIZE). 
pub(crate) fn iv_fill(buffer: &mut [u8]) {
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
     (u32::from_be_bytes(data[0..4].try_into().unwrap()),
      u32::from_be_bytes(data[4..8].try_into().unwrap()))
}

pub(crate) fn block_to_bytes(block: (u32, u32), data: &mut [u8]) {
    data[..4].copy_from_slice(&block.0.to_be_bytes());
    data[4..].copy_from_slice(&block.1.to_be_bytes());
}

pub(crate) fn bytes_to_block3(data: &[u8]) -> (u32,u32,u32) {
    (u32::from_be_bytes(data[0..4].try_into().unwrap()),
     u32::from_be_bytes(data[4..8].try_into().unwrap()),
     u32::from_be_bytes(data[8..12].try_into().unwrap()))
}

pub(crate) fn block3_to_bytes(block: (u32, u32, u32), data: &mut [u8]) {
    data[..4].copy_from_slice(&block.0.to_be_bytes());
    data[4..8].copy_from_slice(&block.1.to_be_bytes());
    data[8..].copy_from_slice(&block.2.to_be_bytes());
}