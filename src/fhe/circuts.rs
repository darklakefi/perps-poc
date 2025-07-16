use crate::key_gen;
use crate::liqudation::users::User;
use tfhe::{
    FheUint64,
    CompressedCiphertextListBuilder,
    set_server_key,
};

pub fn deposit_circuit(user User,amount: u64) -> Result<(), Box<dyn std::error::Error>> {
    let client_key = key_gen::load_client_key()?;
    let server_key = key_gen::load_server_key()?;
    set_server_key((*server_key).clone());
    let value = FheUint64::encrypt(payload.value, &*client_key);
}   

pub fn open_position_circuit(amount: u64) -> Result<(), Box<dyn std::error::Error>> {
    
}

pub fn health_check_circuit() -> Result<(), Box<dyn std::error::Error>> {
    let client_key = key_gen::load_client_key()?;
    let server_key = key_gen::load_server_key()?;
}