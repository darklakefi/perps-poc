use crate::liqudation::users::User;
use serde::{Deserialize, Serialize};
use axum::{Json, http::StatusCode, extract::State};
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

pub async fn encrypt_handler(
    State(state): State<AppState>,
    Json(payload): Json<EncryptRequest>
) -> (StatusCode, Json<EncryptResponse>) {
    let random_bytes: [u8; 32] = rand::random();
    let hold_ciphertext = FheUint64::encrypt(payload.amount, &*state.client_key);
    state.ciphertext_cache.lock().await.add_ciphertext(random_bytes, payload.user_id, hold_ciphertext);
    (StatusCode::OK, Json(EncryptResponse { ciphertext: random_bytes }))
}





