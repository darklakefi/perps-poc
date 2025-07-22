use crate::liqudation::users::User;
use serde::{Deserialize, Serialize};
use axum::{Json, http::StatusCode, extract::{State, Path}};
use crate::AppState;
use rand::Rng;
use crate::fhe::circuits::{health_check_long_circuit, funding_rate_long_pay_short_circuit};
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

#[derive(Deserialize)]
pub struct HealthCheckRequest {
    pub position_id: u128,
    pub mark_price: u64,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
}

#[derive(Deserialize)]
pub struct FundingRateLPSRequest {
    pub position_id: u128,
    pub delta_percent: u64,
}

#[derive(Serialize)]
pub struct FundingRateLPSResponse {
    pub status: String,
}





//////////////////////////////////////////////////////////// Handlers ////////////////////////////////////////////////////////////

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

pub async fn health_check_long_handler(
    State(state): State<AppState>,
    Json(payload): Json<HealthCheckRequest>
) -> (StatusCode, Json<HealthCheckResponse>) { //for now lets just check the first long position in the array
    let position = state.position_cache.lock().await.get_position(0, true).unwrap().clone(); // just get first for now
    let liqdation_price_ciphertext = state.ciphertext_cache.lock().await.get_ciphertext(position.liqudation_price).unwrap().ciphertext.clone();
    let result = health_check_long_circuit(&state, liqdation_price_ciphertext, payload.mark_price).await;
    if result { 
        (StatusCode::OK, Json(HealthCheckResponse { status: "Solvent".to_string() }))
    } else {
        (StatusCode::BAD_REQUEST, Json(HealthCheckResponse { status: "Insolvent".to_string() }))
    }
} 

pub async fn funding_rate_long_pay_short_handler(
    State(state): State<AppState>,
    Json(payload): Json<FundingRateLPSRequest>
) -> (StatusCode, Json<FundingRateLPSResponse>) {
    let position = state.position_cache.lock().await.get_position(payload.position_id, true).unwrap().clone();
    let delta = position.notional * payload.delta_percent / 100; 
    let result = funding_rate_long_pay_short_circuit(
        &state, 
        state.ciphertext_cache.lock().await.get_ciphertext(position.liqudation_price).unwrap().clone(),
        delta).await;
    
    (StatusCode::OK, Json(FundingRateLPSResponse { status: "Success".to_string() }))
}



