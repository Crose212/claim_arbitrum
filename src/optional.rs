use std::env;

use web3::types::U64;
use web3::{Web3,transports::WebSocket};

pub async fn wait_for_block() {

    dotenv::dotenv().ok();
    let websocket = WebSocket::new(&env::var("ETH_SOCKET").unwrap()).await.unwrap();
    let web3s = Web3::new(websocket);

    let needed_block = U64::from(16890400);
    loop {
        let curr_block = web3s.eth().block_number().await.unwrap();
        if needed_block < curr_block {
            println!("Claim not started yet, current block: {:?}", curr_block);
        } else {
            println!("Claim started");
            break;
        }
    }
}
