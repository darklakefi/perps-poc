use crate::liqudation::users::User;
use serde::{Deserialize, Serialize};
use axum::{Json, http::StatusCode, extract::{State, Path}};
use crate::AppState;
use rand::Rng;

use tfhe::{
    FheUint64,
    CompressedCiphertextListBuilder,
    set_server_key,
};
use tfhe::prelude::*;


#[derive(Deserialize)]
pub struct EncryptRequest {
    pub user_id: u128,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct EncryptResponse {
    pub ciphertext: [u8;32],
}

#[derive(Serialize)]
pub struct GetCiphertextResponse {
    pub ciphertext: FheUint64,
}

pub async fn encrypt_handler(
    State(state): State<AppState>,
    Json(payload): Json<EncryptRequest>
) -> (StatusCode, Json<EncryptResponse>) {
    let random_bytes: [u8; 32] = rand::random();
    let hold_ciphertext = FheUint64::encrypt(payload.amount, &*state.client_key);
    state.ciphertext_cache.lock().await.add_ciphertext(random_bytes, payload.user_id, hold_ciphertext);
    (StatusCode::OK, Json(EncryptResponse { ciphertext: random_bytes }))
}

pub async fn _encrypt_helper(State(state): State<AppState>, amount: u64, user_id: u128) -> [u8;32] {
    let random_bytes: [u8; 32] = rand::random();
    let hold_ciphertext = FheUint64::encrypt(amount, &*state.client_key);
    state.ciphertext_cache.lock().await.add_ciphertext(random_bytes, user_id, hold_ciphertext);
    random_bytes
}

pub async fn _encrypt_from_FheUint64(State(state): State<AppState>, amount: FheUint64, user_id: u128) -> [u8;32] {
    let random_bytes: [u8; 32] = rand::random();
    let hold_ciphertext = amount;
    state.ciphertext_cache.lock().await.add_ciphertext(random_bytes, user_id, hold_ciphertext);
    random_bytes
}

pub async fn get_ciphertext_handler(
    State(state): State<AppState>,
    Path(ciphertext_key): Path<[u8;32]>
) -> (StatusCode, Json<GetCiphertextResponse>) {
    let ciphertext = state.ciphertext_cache.lock().await.get_ciphertext(ciphertext_key).unwrap().clone();
    (StatusCode::OK, Json(GetCiphertextResponse { ciphertext: ciphertext.ciphertext.clone() }))
}





