#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental

use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{Impl, Sha256};

risc0_zkvm::guest::entry!(main);
extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

// headers pain & proof of quark

fn assert_le_32(time: [u8; 4], time_prev_block: &[u8]) {

    for n in 0..4 {

        let index = 3 - n; 
        
        if time_prev_block[index] == time[index] { continue }
        
        if time[index] < time_prev_block[index] { break }
        
        panic!("Time is not on my side");
    }
}

fn assert_eq_32(version: [u8; 4], header_version: &[u8]) {

    for n in 0..4 { assert_eq!(header_version[n], version[n]) }
}

// nBits = 0x1d00ffff

fn bits_to_target(n_bits: &[u8]) -> [u8; 32] {

    let exponent: usize = n_bits[3] as usize;

    assert_ne!(exponent, 0);
    assert_ne!(exponent, 1);
    assert_ne!(exponent, 2);

    let mut target = [0u8; 32];

    target[exponent - 3 + 0] = n_bits[0];
    target[exponent - 3 + 1] = n_bits[1];
    target[exponent - 3 + 2] = n_bits[2];

    return target;
}

fn assert_le_256(block_header_hash: &[u8], target: [u8; 32]) {

    for n in 0..32 {

        let index = 31 - n; 
        
        if target[index] == block_header_hash[index] { continue }
        
        if block_header_hash[index] < target[index] { break }
        
        panic!("Insufficient proof of work");
    }
}

fn assert_eq_256(hash_prev_block: [u8; 32], header_hash_prev_block: &[u8]) {

    for n in 0..32 { assert_eq!(header_hash_prev_block[n], hash_prev_block[n]) }
}
fn median(block_times: &mut VecDeque<u64>) -> Option<f64> {
    let mut sorted = block_times.clone().into_iter().collect::<Vec<_>>();
    sorted.sort();
    let len = sorted.len();
    if len % 2 ==1 {
        // If the length is odd, return the middle value
        Some(sorted[len / 2] as f64)
    } else {
         // If the length is even, return the average of the two middle values
         let first_mid = sorted[len / 2 - 1] as f64;
         let second_mid = sorted[len / 2] as f64;
         Some((first_mid + second_mid) / 2.0)
    }

}
pub fn main() {

    // Receive the number of block headers to verify
    let num_blocks: u32 = env::read();

    // Genesis block has hashPrevBlock = zero
    let mut hash_prev_block = [0u8; 32];

    // 03/Jan/2009
    let mut time_prev_block: [u8; 4] = [0x40, 0x53, 0x5f, 0x49];
     // Store block time
     let mut block_times: VecDeque<u64> = VecDeque::with_capacity(11);  // Assuming median of 11 blocks
    
    for block_height in 0..num_blocks {

        // Receive next block header
        let mut header = [0u8; 80];
        env::read_slice(&mut header);

        // Checking version
        assert_eq_32([1u8, 0, 0, 0], &header[0..4]);
        
        // Checking previous block header hash
        assert_eq_256(hash_prev_block, &header[4..36]);

        // Computing block header hash
        let sha256 = Impl::hash_bytes(&header);
        let hash256 = Impl::hash_bytes(&sha256.as_bytes());

        // Checking unix epoch time
        let time: &[u8] = &header[68..72];
        assert_le_32(time_prev_block, time);
        time_prev_block.copy_from_slice(time);

        // Checking proof-of-work against target
        let target = bits_to_target(&header[72..76]);
        assert_le_256(hash256.as_bytes(), target);

        // Commit to final block header hash
        if block_height == num_blocks-1 { env::commit(&hash256.as_bytes()); }

        // Or memorise block header hash (for the next block)
        else { hash_prev_block.copy_from_slice(&hash256.as_bytes()); }
    //    check unix epoch time
    let time: &[u8] = &header[68..72];
    let time_as_u64 = u64::from_le_bytes([time[0], time[1], time[2], time[3], 0, 0, 0, 0]);

    if block_times.len() == 11 {
        block_times.pop_front();

    }
    block_times.push_back(time_as_u64);
    if block_times.len() == 11 {
        let median = median(&mut block_times).unwrap();
        let time_as_f64 = time_as_u64 as f64;
        if time_as_f64 > median * 2.0 {
            panic!("Block time is too far in the future");
        }
    }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
    


    #[test]
    fn test_median_odd() {
        let mut block_times: VecDeque<u64> = VecDeque::from(vec![1, 3, 2, 4, 5]);
        assert_eq!(median(&mut block_times), Some(3.0));
    }

    #[test]
    fn test_median_even() {
        let mut block_times: VecDeque<u64> = VecDeque::from(vec![1, 3, 2, 4]);
        assert_eq!(median(&mut block_times), Some(2.5));
    }

    #[test]
    fn test_median_empty() {
        let mut block_times: VecDeque<u64> = VecDeque::new();
        assert_eq!(median(&mut block_times), None);
    }
}