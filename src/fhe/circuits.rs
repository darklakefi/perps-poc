use crate::liqudation::users::User;
use tfhe::{
    FheUint64,
    CompressedCiphertextListBuilder,
    set_server_key,
};
use tfhe::prelude::*;
use crate::AppState;

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

