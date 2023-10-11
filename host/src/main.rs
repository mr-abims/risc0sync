// TODO: Update the name of the method loaded by the prover. E.g., if the method
// is `multiply`, replace `METHOD_NAME_ELF` with `MULTIPLY_ELF` and replace
// `METHOD_NAME_ID` with `MULTIPLY_ID`
use methods::{METHOD_NAME_ELF, METHOD_NAME_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

const NUM_BLOCKS: u32 = 30;

fn main() {

    let mut env_builder = ExecutorEnv::builder();

    // Send the number of block headers to verify
    env_builder.add_input(&[NUM_BLOCKS]);

    // Fetch and send all block headers
    for block_height in 0..NUM_BLOCKS {

        // Fetch block header
        let s_block_id = reqwest::blocking::get(format!("https://blockstream.info/api/block-height/{}", block_height)).unwrap().text().unwrap();
        let s_header = reqwest::blocking::get(format!("https://blockstream.info/api/block/{}/header", s_block_id)).unwrap().text().unwrap();
        
        let mut header = [0u8; 80];
        for (hex8, byte) in s_header.chars().collect::<Vec<char>>().chunks(2).map(|c| c.iter().collect::<String>()).zip(header.iter_mut()) {
            *byte = u8::from_str_radix(&hex8, 16).unwrap();
        }

        // Send block header
        env_builder.add_input(&header);
    }

    let env = env_builder.build().unwrap();

    println!("Received {} block headers from blockstream.info", NUM_BLOCKS);

    let prover = default_prover();

    let receipt = prover.prove_elf(env, METHOD_NAME_ELF).unwrap();

    receipt.verify(METHOD_NAME_ID).unwrap();

    let digest = receipt.journal.chunks(4).rev().map(|word| format!("{:02x?}", word[0])).collect::<Vec<_>>().join("");

    println!("The {}th block has hashPrevBlock = {}", NUM_BLOCKS, digest);
}
