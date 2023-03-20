use web3::types::U64;
use web3::{Web3,transports::WebSocket};

pub async fn wait_for_block(web3s: Web3<WebSocket>) {
    let needed_block = U64::from(16890400);
    loop {
        let curr_block = web3s.eth().block_number().await.unwrap();
        if needed_block != curr_block {
            println!("Claim not started yet");
        } else {
            println!("Claim started");
            break;
        }
    }
}
