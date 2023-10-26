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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Once, ONCE_INIT};

    static START: Once = ONCE_INIT;

    fn setup() {
        START.call_once(|| {
            // Any setup code you might need, like mocking endpoints, should be placed here
        });
    }

    // Mock for reqwest
    fn mock_get(url: &str) -> Result<String, reqwest::Error> {
        // Here we return mock data based on the input URL
        match url {
            "https://blockstream.info/api/block-height/0" => Ok("mock_block_id_0".to_string()),
            "https://blockstream.info/api/block/0/header" => Ok("mock_header_0".to_string()),
            // ... continue this pattern for all blocks up to `NUM_BLOCKS`
            _ => Err(reqwest::Error::new(reqwest::StatusCode::NOT_FOUND, "not found")),
        }
    }

    #[test]
    fn test_fetch_and_verify_block_headers() {
        setup();
        
        // Capture the output of your main function using a hook
        let output = std::io::Cursor::new(Vec::new());
        let old_stdout = std::io::stdout();
        std::io::set_stdout(Box::new(output));

        main(); // Execute the main function
        
        // Reset stdout and get the captured output
        std::io::set_stdout(old_stdout);
        let output = String::from_utf8(output.into_inner()).unwrap();

        // Check that the expected block headers were processed and the output matches our expectations
        assert!(output.contains(&format!("Received {} block headers from blockstream.info", NUM_BLOCKS)));
        assert!(output.contains(&format!("The {}th block has hashPrevBlock =", NUM_BLOCKS)));
    }