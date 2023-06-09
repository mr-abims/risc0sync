use methods::{HEADERS_CHAIN_PROOF_ELF, HEADERS_CHAIN_PROOF_ID};
use risc0_zkp::core::sha::Digest;
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Prover;

const NUM_BLOCKS: u8 = 30;

fn main() {

    let mut prover = Prover::new(HEADERS_CHAIN_PROOF_ELF, HEADERS_CHAIN_PROOF_ID).unwrap();

    // Send the number of block headers to verify
    prover.add_input_u32_slice(&to_vec(&format!("{:02X?}", NUM_BLOCKS)).unwrap());

    // Fetch and send all block headers
    for block_height in 0..NUM_BLOCKS {

        // Fetch block header
        let block_id = reqwest::blocking::get(format!("https://blockstream.info/api/block-height/{}", block_height)).unwrap().text().unwrap();
        let header = reqwest::blocking::get(format!("https://blockstream.info/api/block/{}/header", block_id)).unwrap().text().unwrap();
        
        // Send block header
        prover.add_input_u32_slice(&to_vec(&header).unwrap());
    }

    println!("Received {} block headers from blockstream.info", NUM_BLOCKS);

    let receipt = prover.run().unwrap();

    receipt.verify(HEADERS_CHAIN_PROOF_ID).unwrap();

    let digest = from_slice::<Digest>(receipt.journal.as_slice()).unwrap();

    println!("The {}th block has hashPrevBlock = {}", NUM_BLOCKS, digest);
}
