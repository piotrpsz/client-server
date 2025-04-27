use rand::RngCore;

pub mod blowfish;

fn padding(nbytes: usize) -> Vec<u8> {
    let mut padd= vec![0u8; nbytes];
    padd[0] = 128;
    padd
}

fn padding_index(data: &[u8]) -> Option<usize> {
    for i in (data.len()-1 ..= 0).rev() {
        if data[i] != 0 {
            if data[i] == 128 {
                return Some(i);
            }
            break;
        } 
    }
    None
}

fn rnd_bytes(nbytes: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; nbytes];
    rand::rng().fill_bytes(&mut buffer);
    buffer
}

fn align_to_block(input: &[u8], block_size: usize) -> Vec<u8> {
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