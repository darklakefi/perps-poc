use crate::liqudation::users::User;
use serde::{Deserialize, Serialize};
use axum::{Json, http::StatusCode, extract::State};
use crate::AppState;
use rand::Rng;


#[derive(Deserialize)]
pub struct EncryptRequest {
    pub user_id: u128,
    pub amount: [u8;32],
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
    (StatusCode::OK, Json(EncryptResponse { ciphertext: random_bytes }))
}



