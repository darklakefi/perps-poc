use crate::liqudation::users::User;
use crate::State;
use tfhe::{
    FheUint64,
    CompressedCiphertextListBuilder,
    set_server_key,
};
use tfhe::prelude::*;
use crate::AppState;
use crate::liqudation::users::Position;
use crate::liqudation::handlers::{_encrypt_helper, _encrypt_from_FheUint64};
use crate::liqudation::cache::Ciphertext;

pub async fn deposit_circuit(state: &AppState, user_id: u128, amount: u64, key: [u8;32]) -> Result<(), Box<dyn std::error::Error>> {
    set_server_key((*state.server_key).clone());
    println!("Attempting to deposit");
    let value = FheUint64::encrypt(amount, &*state.client_key); // generates the actual ciphertext
    if state.user_cache.lock().await.get_user(user_id).unwrap().balance == [0;32] { // if the user has no balance yet 
        state.ciphertext_cache.lock().await.add_ciphertext(key, user_id, value); //adds this ciphertext 
        state.user_cache.lock().await.update_balance(user_id, key);
        // TODO thread to write this to db 
    } else { // if they already have a balance then we need to add the new amount to the existing balance
        let current_balance_key = state.user_cache.lock().await.get_user(user_id).unwrap().balance;
        let current_balance_ciphertext = state.ciphertext_cache.lock().await.get_ciphertext(current_balance_key).unwrap().ciphertext.clone();
        let new_balance_ciphertext = &current_balance_ciphertext + &value;
        
        // Update the ciphertext in the cache
        state.ciphertext_cache.lock().await.update_ciphertext(current_balance_key, user_id, new_balance_ciphertext);
        // TODO thread to write this to db 
    }
    println!("Deposit successful");
    Ok(())
}   


async fn withdraw_circuit(state: &AppState, user_id: u128, amount: u64, key:[u8;32]) -> Result<(), Box<dyn std::error::Error>> {
    set_server_key((*state.server_key).clone());
    Ok(())
}

pub async fn open_position_circuit(
    state: &AppState,
    user_id: u128,
    entry_price: u64,
    direction: bool,
    notional: u64,
    leverage_ciphertext: FheUint64,
    initial_margin_ciphertext: FheUint64,
    initial_margin_key: [u8;32],
    leverage_key: [u8;32],
) -> Result<(), Box<dyn std::error::Error>> { // chage this back to position after i finish testing ******
    let start_time = std::time::Instant::now();
    println!("[{}ms] Opening position...", start_time.elapsed().as_millis());
    
    let opening_fee = (notional as f64 * 0.01).ceil() as u64; // for lets assume this is the same as margin as well, ill use a constant later
    set_server_key((*state.server_key).clone());
    println!("[{}ms] Server key set", start_time.elapsed().as_millis());
    
    let opening_fee_ciphertext = FheUint64::encrypt(opening_fee, &*state.client_key); // use some sort of looh up later for these to avoid encryption
    println!("[{}ms] Opening fee encrypted", start_time.elapsed().as_millis());
    
    let notional_ciphertext = FheUint64::encrypt(notional, &*state.client_key);
    println!("[{}ms] Notional encrypted", start_time.elapsed().as_millis());
    
    // let valid_notional = notional_ciphertext.eq(&initial_margin_ciphertext * &leverage_ciphertext);// check for valid notional
    // println!("[{}ms] Notional validation computed", start_time.elapsed().as_millis());
    
    // let notional_decrypted = &valid_notional.decrypt(&*state.client_key);
    // println!("[{}ms] Notional validation decrypted", start_time.elapsed().as_millis());
    let notional_decrypted = true;
    
    if notional_decrypted {
        let liqudation_price_ciphertext = notional_ciphertext - opening_fee_ciphertext - initial_margin_ciphertext.clone(); // prob need to adjust this later  
        println!("[{}ms] Liquidation price computed", start_time.elapsed().as_millis());
        
        let liqudation_price_key = _encrypt_from_FheUint64(State(state.clone()), liqudation_price_ciphertext, user_id).await;
        println!("[{}ms] Liquidation price encrypted and stored", start_time.elapsed().as_millis());
        
        // need to create the actual ciphertext for liqudation price 
        let hold_position = Position {
            id: state.position_cache.lock().await.n,
            direction,
            notional,
            entry_price: entry_price,
            leverage: leverage_key,
            initial_margin: initial_margin_key,
            liqudation_price: liqudation_price_key,
        };
        println!("[{}ms] Position object created", start_time.elapsed().as_millis());
        
        // need to deduct intiial margin from user balance
        let current_balance_key = state.user_cache.lock().await.get_user(user_id).unwrap().balance;
        let current_balance_ciphertext = state.ciphertext_cache.lock().await.get_ciphertext(current_balance_key).unwrap().ciphertext.clone();
        let new_balance_ciphertext = &current_balance_ciphertext - &initial_margin_ciphertext;
        println!("[{}ms] New balance computed", start_time.elapsed().as_millis());
        
        state.ciphertext_cache.lock().await.update_ciphertext(current_balance_key, user_id, new_balance_ciphertext); // update the ciphertext balance
        println!("[{}ms] Balance updated in cache", start_time.elapsed().as_millis());
        
        state.user_cache.lock().await.add_position(user_id, hold_position.clone()); // add the position to the user_cache array
        println!("[{}ms] Position added to user cache", start_time.elapsed().as_millis());
        
        state.position_cache.lock().await.add_position(hold_position); // add the position to the cache
        println!("[{}ms] Position added to position cache", start_time.elapsed().as_millis());

        println!("[{}ms] Position opened successfully!", start_time.elapsed().as_millis());
        Ok(())
    } else {
        println!("[{}ms] Invalid notional - position opening failed", start_time.elapsed().as_millis());
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Invalid notional")));
    }
}

pub async fn health_check_long_circuit(state: &AppState, liqdation_price: FheUint64, mark_price: u64) -> bool {
    set_server_key((*state.server_key).clone());
    println!("Health check long circuit called");
    let mark_ciphertext = FheUint64::encrypt(mark_price, &*state.client_key);
    let status_ciphertext = mark_ciphertext.ge(&liqdation_price);
    let status_decrypted = status_ciphertext.decrypt(&*state.client_key);
    status_decrypted
}

pub async fn funding_rate_long_pay_short_circuit(state: &AppState, liqudation_price_ciphertext: Ciphertext, delta: u64) -> Result<(), Box<dyn std::error::Error>> {
    set_server_key((*state.server_key).clone());
    let encrypted_delta = FheUint64::encrypt(delta, &*state.client_key).clone();
    println!("trying to do the math for funding rate LPS");
    
    let decrypted_delta: u64 = encrypted_delta.decrypt(&*state.client_key);
    println!("delta decrypted: {}", decrypted_delta);
    let decrypted_liqdation_price: u64 = liqudation_price_ciphertext.ciphertext.decrypt(&*state.client_key);
    println!("liqdation price decrypted: {}", decrypted_liqdation_price);


    let new_liqdation_price_ciphertext = &liqudation_price_ciphertext.ciphertext - &encrypted_delta;
    println!("new liqdation price computed");
    let update_result = state.ciphertext_cache.lock().await.update_ciphertext(liqudation_price_ciphertext.key, liqudation_price_ciphertext.owner, new_liqdation_price_ciphertext);
    println!("update_ciphertext result: {}", update_result);
    println!("funding rate LPS circuit finished");
    Ok(())
}


