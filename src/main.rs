mod optional;

use optional::wait_untill_unix;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use secp256k1::SecretKey;
use web3::contract::{Contract, tokens::Tokenize};
use web3::{Web3, transports::WebSocket};
use web3::types::{Address, Bytes, TransactionParameters, H160, U256, SignedTransaction};

#[tokio::main]
async fn main() -> web3::Result<()> {

    dotenv::dotenv().ok();

    let websocket = WebSocket::new(&env::var("SOCKET").unwrap()).await?;
    let web3s = Web3::new(websocket);
    println!("connected to WebSocket, current block: {:?}", web3s.eth().block_number().await.unwrap());
    
    let gas_price = web3s.eth().gas_price().await.unwrap();
        println!("gas price in Gwei: {}", gas_price.as_u64() / (u64::pow(10, 9)));
        println!("gas price in Wei: {}", gas_price);

    let private_keys = read_private_keys("./files/pkeys.txt").await;
    let addresses = read_addresses("./files/addresses.txt").await;
    println!("Loaded {:?} addresses and {:?} private keys", addresses.len(), private_keys.len());

    let mut tasks = Vec::new();
    for account in &addresses {
        let task = tokio::task::spawn(load_balances(*account, web3s.clone()));
        tasks.push(task);
    }
    futures::future::join_all(tasks).await;

    let contract_addr = Address::from_str("0x67a24CE4321aB3aF51c2D0a4801c3E111D88C9d9").unwrap();
    let contract = Contract::from_json(
        web3s.eth(),
        contract_addr,
        include_bytes!("contract_abi.json")
    )
    .unwrap();

    //let good_gas = get_good_gas(&contract, addresses[0]).await; // customize func name and params

    let data = contract
        .abi()
        .function("claim")
        .unwrap()
        .encode_input(
            &(
            )
                .into_tokens(),
        )
        .unwrap();

    let signed_trans = get_signed_transactions(addresses, data, contract_addr, private_keys, web3s.clone()).await;

    wait_for_block(web3s.clone()).await;
    
    let mut tasks2 = Vec::new();
    let signed_trans = signed_trans.lock().unwrap();
    for data in signed_trans.iter() {

        let task = tokio::task::spawn(send_trans(data.clone(), web3s.clone()));
        tasks2.push(task);
    }
    futures::future::join_all(tasks2).await;
    
    println!("All transactions have been sent");
    Ok(())
}

async fn get_signed_transactions(addresses: Vec<H160>, data: Vec<u8>, contract_addr: H160, private_keys: Vec<String>, web3s: Web3<WebSocket>) -> Arc<std::sync::Mutex<Vec<SignedTransaction>>> {

    let signed_trans = Arc::new(Mutex::new(vec![]));
    let mut futures = vec![];

    for i in 0..addresses.len() {

        let signable_data = data.clone();
        let nonce = web3s.eth().transaction_count(addresses[i], None);
        let signed_trans = signed_trans.clone();
        let contract_addr = contract_addr.clone();
        let web3s = web3s.clone();
        let private_keys = private_keys.clone();

        let future = async move {

            let nonce = nonce.await.unwrap();
            let transaction_obj = TransactionParameters {
                
                nonce: Some(nonce),
                to: Some(contract_addr),
                value: U256::exp10(14) * 0, //0.0001 eth * N
                gas: U256::exp10(5) * 20, // 100_000 * N
                //gas_price: Some(U256::exp10(9) * 5),  // 1 gwei * N
                gas_price: Some(web3s.eth().gas_price().await.unwrap()),
                data: Bytes(signable_data),
                ..Default::default()
            };
            let secret = SecretKey::from_str(&private_keys[i].to_string()).unwrap();
            let signed_data = web3s
                .accounts()
                .sign_transaction(transaction_obj, &secret)
                .await
                .unwrap();

            let mut signed_trans = signed_trans.lock().unwrap();
            signed_trans.push(signed_data);
        };
        futures.push(tokio::spawn(future));
        println!("Signed transaction for account: {:?}", &addresses[i]);
    }
    futures::future::join_all(futures).await;
    signed_trans
}

async fn send_trans(data: SignedTransaction, web3s: Web3<WebSocket>) {


    let result = web3s
        .eth()
        .send_raw_transaction(data.raw_transaction)
        .await
        .unwrap();

    let curr_block = web3s.eth().block_number().await.unwrap();

    println!("Transaction sent with hash: {:?}, block: {:?}", result, curr_block);
    std::thread::sleep(Duration::from_millis(60000));
    loop {
        
        if web3s.eth().block_number().await.unwrap() != curr_block {
            println!("block mined");
            break;
        }
        std::thread::sleep(Duration::from_millis(2000));
        println!("sleeping...");
    }
    let transaction_receipt = web3s.eth().transaction_receipt(result).await.unwrap().unwrap_or_default();
    println!("Transaction mined: {:?}", transaction_receipt);
}

async fn load_balances(address: H160, web3s: Web3<WebSocket>) -> Option<U256>{

    let wei_conv: U256 = U256::exp10(13);

    let balance = web3s.eth().balance(address, None).await.unwrap();
    println!("Eth balance of {:?}: {}", address, balance.checked_div(wei_conv).unwrap());
    Some(balance)
}

async fn read_private_keys(file_path: &str) -> Vec<String> {

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut private_keys = Vec::new();

    for line in reader.lines() {

        let private_key = line.unwrap().to_string();
        private_keys.push(private_key);
    }
    private_keys
}

async fn read_addresses(file_path: &str) -> Vec<H160> {

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut addresses = Vec::new();

    for line in reader.lines() {

        let line = line.unwrap();
        let address = H160::from_str(&line).unwrap();
        addresses.push(address);
    }
    addresses
}
