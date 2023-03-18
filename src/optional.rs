use std::time::{SystemTime, UNIX_EPOCH};
use gstuff::duration_to_float;


#[allow(dead_code)]
pub async fn wait_untill_unix() {
    
    let claim_time = 1689040000_f64;
    loop {
        let timestamp: f64 = duration_to_float(SystemTime::now()    
    .duration_since(UNIX_EPOCH).expect("async time"));
    
        if timestamp >= claim_time {
                println!("Claim is live");
            break;
        }
        
        println!("time left untill claim: {:?}" , claim_time - timestamp )
    }

}
