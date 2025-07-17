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
    let value = FheUint64::encrypt(amount, &*state.client_key); // generates the actual ciphertext
    if state.user_cache.lock().await.get_user(user_id).unwrap().balance == [0;32] { // if the user has no balance yet 
        state.ciphertext_cache.lock().await.add_ciphertext(key, user_id, value);

        // TODO thread to write this to db
    } else {

    }
    


    Ok(())
}   

