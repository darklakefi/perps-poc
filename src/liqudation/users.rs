use axum::{Json, http::StatusCode, extract::State};
use serde::{Deserialize, Serialize};
use crate::liqudation::cache::{AccountCache, SharedAccountCache, CiphertextCache};
use crate::fhe::circuits::{deposit_circuit};
use tfhe::prelude::FheDecrypt;
use tfhe::set_server_key;
use axum::extract::Path;
use crate::AppState;


#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: u128,
    pub notional: u128,
    pub direction: bool,
    pub entry_price: u128,
    pub leverage: [u8;32],
    pub initial_maring: [u8;32],
    pub liqudation_price: [u8;32],
}

#[derive(Clone)]    
pub struct User {
    pub id: u128,
    pub positions: Vec<Position>,
    pub balance: [u8;32],
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    user_id: u128,
}

#[derive(Deserialize)]
pub struct GetUserRequest {
    user_id: u128,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    user_id: u128,
    message: String,
}

#[derive(Serialize)]
pub struct GetUserResponse {
    user_id: u128,
    positions: Vec<Position>,
    balance: [u8;32],
}

#[derive(Deserialize)]
pub struct DepositRequest {
    user_id: u128,
    amount: u64,
    key: [u8;32],
}

#[derive(Serialize)]
pub struct DepositResponse {
    message: String,
}


pub fn create_user(id: u128) -> User {
    User {
        id,
        positions: Vec::new(),
        balance: [0;32],
    }
}

#[derive(Deserialize)]
pub struct ViewBalanceRequest {
    user_id: u128,
}

#[derive(Serialize)]
pub struct ViewBalanceResponse {
    plaintext: u64,
}

////////////////////////////// Handlers //////////////////////////////

#[axum::debug_handler]
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>
) -> (StatusCode, Json<CreateUserResponse>) {
    let user = create_user(payload.user_id);
    // TODO: Store user in database/state
    
    let mut cache_guard = state.user_cache.lock().await;
    let success = cache_guard.add_user(user);

    let response = if success {
        CreateUserResponse {
            user_id: payload.user_id,
            message: "User created successfully".to_string(),
        }
    } else {
        CreateUserResponse {
            user_id: payload.user_id,
            message: "User already exists".to_string(),
        }
    };
    
    let status = if success { StatusCode::CREATED } else { StatusCode::CONFLICT };
    (status, Json(response))
}


#[axum::debug_handler]
pub async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<u128>
) -> (StatusCode, Json<GetUserResponse>) {
    let mut cache_guard = state.user_cache.lock().await;
    let user = cache_guard.get_user(user_id);

    let response = if let Some(user) = user {
        GetUserResponse {
            user_id,
            positions: user.positions.clone(),
            balance: user.balance.clone(),
        }
    } else {
        GetUserResponse {
            user_id,
            positions: vec![],
            balance: [0; 32],
        }
    };

    (StatusCode::OK, Json(response))
}

#[axum::debug_handler]
pub async fn deposit_handler(
    State(state): State<AppState>,
    Json(payload): Json<DepositRequest>
) -> (StatusCode, Json<DepositResponse>) {

    deposit_circuit(&state, payload.user_id, payload.amount, payload.key).await;
    (StatusCode::OK, Json(DepositResponse { message: "Deposit successful".to_string() }))
}

pub async fn view_balance_handler(
    State(state): State<AppState>,
    Path(user_id): Path<u128>
) -> (StatusCode, Json<ViewBalanceResponse>) {
    set_server_key((*state.server_key).clone());
    let balance = state.user_cache.lock().await.get_balance(user_id).unwrap().clone();
    let decrypted: u64 = state.ciphertext_cache.lock().await.get_ciphertext(balance).unwrap().ciphertext.decrypt(&state.client_key);
    let response = ViewBalanceResponse {
        plaintext: decrypted,
    };
    (StatusCode::OK, Json(response))
}