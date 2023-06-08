
// headers pain & proof of quark

#![no_main]

use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{sha, Sha};

risc0_zkvm::guest::entry!(main);

// nBits = 0x1d00ffff

// fn bits_to_target(nBits: u32) -> [u8; 32] {
//     let exponent = nBits >> 24;
//     let significand = nBits & 0x00ffffff;

//     let shift = 1 << (8 * (exponent - 3));
//     let target = significand * shift;
// }

pub fn main() {

    // Receive the number of block headers to verify
    let num_blocks: u8 = hex::decode(&env::read::<String>()).unwrap()[0];
    let mut hash_prev_block = [0u8; 32];
    for block_height in 0..num_blocks {

        // Receive next block header
        let header: Vec<u8> = hex::decode(&env::read::<String>()).unwrap();
        // let txid: Vec<u8> = hex::decode(&env::read::<String>()).unwrap();
        // let tx: Vec<u8> = hex::decode(&env::read::<String>()).unwrap();

        // Checking block headers length
        if header.len() != 80 {
            panic!("block header is 80 byte-sized");
        }

        // if genesis_block.len() != 285 {
        //     panic!("genesis block is 285 byte-sized");
        // }

        // Checking version
        assert_eq!(header[0], 1);
        assert_eq!(header[1], 0);
        assert_eq!(header[2], 0);
        assert_eq!(header[3], 0);
        
        // Checking previous block header hash
        for n in 0..32 {
            assert_eq!(hash_prev_block[n], header[n+4]);
        }

        // let sha256 = sha().hash_bytes(&tx);
        // let hash256 = sha().hash_bytes(&sha256.as_bytes());
        // for n in 0..32 {
            // assert_eq!(hash256.as_bytes()[n], header[n+36]);
        // }

        // Computing block header hash
        let sha256 = sha().hash_bytes(&header);
        let hash256 = sha().hash_bytes(&sha256.as_bytes());

        // Commit to final block header hash
        if block_height == num_blocks-1 { env::commit(&*hash256); }

        // Or memorise block header hash (for the next block)
        else { hash_prev_block.copy_from_slice(&hash256.as_bytes()); }
    }
}
