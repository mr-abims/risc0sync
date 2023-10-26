#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental

use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{Impl, Sha256};

risc0_zkvm::guest::entry!(main);

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

pub fn main() {

    // Receive the number of block headers to verify
    let num_blocks: u32 = env::read();

    // Genesis block has hashPrevBlock = zero
    let mut hash_prev_block = [0u8; 32];

    // 03/Jan/2009
    let mut time_prev_block: [u8; 4] = [0x40, 0x53, 0x5f, 0x49];
    
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
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use reqwest;
    use zkvm_env::env;

    const BASE_URL: &str = "https://blockstream.info/api";

    fn fetch_block_height() -> Result<u32, reqwest::Error> {
        let response: u32 = reqwest::blocking::get(&format!("{}/block-height/", BASE_URL))?.json()?;
        Ok(response)
    }

    fn fetch_block_header(block_height: u32) -> Result<[u8; 80], reqwest::Error> {
        let response: Vec<u8> = reqwest::blocking::get(&format!("{}/block/{}/header", BASE_URL, block_height))?.bytes()?.to_vec();
        let mut header = [0u8; 80];
        header.copy_from_slice(&response);
        Ok(header)
    }

    #[test]
    fn test_main_valid_block() {
        let block_height = fetch_block_height().expect("Failed to fetch block height");

        let block_header = fetch_block_header(block_height).expect("Failed to fetch block header");

        env::set_test_data(block_height, block_header);

        // Run the main function
        main();

       
        let committed_data = env::get_committed_data();
        assert!(committed_data.is_some());
        let committed_data_ref = committed_data.as_ref().unwrap();

        // Compute expected commit value based on fetched header
        let sha256 = Impl::hash_bytes(&block_header);
        let expected_commit_data = Impl::hash_bytes(&sha256.as_bytes());

        assert_eq!(committed_data_ref, &expected_commit_data.as_bytes());
    }
}

mod zkvm_env {
    #[cfg(not(test))]
    pub use risc0_zkvm::guest::env::*;

    #[cfg(test)]
    pub mod env {
        static mut TEST_BLOCK_HEIGHT: Option<u32> = None;
        static mut TEST_HEADER: Option<[u8; 80]> = None;
        static mut COMMITTED_DATA: Option<Vec<u8>> = None;

        pub fn read() -> u32 {
            unsafe {
                TEST_BLOCK_HEIGHT.expect("Block height not set for test")
            }
        }

        pub fn read_slice(buffer: &mut [u8]) {
            unsafe {
                buffer.copy_from_slice(&TEST_HEADER.expect("Header not set for test"));
            }
        }

        pub fn commit(data: &[u8]) {
            unsafe {
                COMMITTED_DATA = Some(data.to_vec());
            }
        }

        pub fn get_committed_data() -> Option<Vec<u8>> {
            unsafe { COMMITTED_DATA.clone() }
        }

        pub fn set_test_data(block_height: u32, header: [u8; 80]) {
            unsafe {
                TEST_BLOCK_HEIGHT = Some(block_height);
                TEST_HEADER = Some(header);
            }
        }
    }
}