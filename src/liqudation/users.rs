use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::zyga_proof::verifier::extract_balance_transition;
use crate::AppState;

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: u128,
    pub direction: bool,
    pub notional: u64,
    pub entry_price: u64,
    pub leverage: u64,
    pub initial_margin: u64,
    pub liquidation_price: u64,
}

#[derive(Clone)]
pub struct User {
    pub id: u128,
    pub positions: Vec<Position>,
    pub balance_commitment: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
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
    balance_commitment: Option<String>,
}

#[derive(Deserialize)]
pub struct DepositRequest {
    user_id: u128,
    amount: u64,
    proof: serde_json::Value,
}

#[derive(Serialize)]
pub struct DepositResponse {
    message: String,
    new_balance_commitment: String,
}

#[derive(Serialize)]
pub struct ViewBalanceResponse {
    balance_commitment: Option<String>,
}

pub fn create_user(id: u128) -> User {
    User {
        id,
        positions: Vec::new(),
        balance_commitment: None,
    }
}

#[derive(Deserialize)]
pub struct OpenPositionRequest {
    user_id: u128,
    direction: bool,
    entry_price: u64,
    notional: u64,
    leverage: u64,
    initial_margin: u64,
    balance_proof: serde_json::Value,
}

#[derive(Serialize)]
pub struct OpenPositionResponse {
    message: String,
    position_id: u128,
    balance_commitment: String,
    liquidation_price: u64,
}

#[axum::debug_handler]
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<CreateUserResponse>) {
    let user = create_user(payload.user_id);
    let success = state.user_cache.lock().await.add_user(user);
    let status = if success {
        StatusCode::CREATED
    } else {
        StatusCode::CONFLICT
    };

    (
        status,
        Json(CreateUserResponse {
            user_id: payload.user_id,
            message: if success {
                "User created successfully".to_string()
            } else {
                "User already exists".to_string()
            },
        }),
    )
}

#[axum::debug_handler]
pub async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<u128>,
) -> (StatusCode, Json<GetUserResponse>) {
    let user = state.user_cache.lock().await.get_user(user_id).cloned();
    (
        StatusCode::OK,
        Json(match user {
            Some(user) => GetUserResponse {
                user_id,
                positions: user.positions,
                balance_commitment: user.balance_commitment,
            },
            None => GetUserResponse {
                user_id,
                positions: vec![],
                balance_commitment: None,
            },
        }),
    )
}

#[axum::debug_handler]
pub async fn deposit_handler(
    State(state): State<AppState>,
    Json(payload): Json<DepositRequest>,
) -> (StatusCode, Json<DepositResponse>) {
    let transition = match extract_balance_transition(&payload.proof, payload.amount) {
        Ok(transition) => transition,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DepositResponse {
                    message,
                    new_balance_commitment: String::new(),
                }),
            )
        }
    };

    let mut user_cache = state.user_cache.lock().await;
    let stored = user_cache.get_balance_commitment(payload.user_id).map(str::to_owned);
    if let Some(stored_commitment) = stored {
        if stored_commitment != transition.old_commitment {
            return (
                StatusCode::BAD_REQUEST,
                Json(DepositResponse {
                    message: "old_commitment does not match stored user commitment".to_string(),
                    new_balance_commitment: String::new(),
                }),
            );
        }
    }

    user_cache.set_balance_commitment(payload.user_id, transition.new_commitment.clone());
    (
        StatusCode::OK,
        Json(DepositResponse {
            message: "Deposit proof accepted".to_string(),
            new_balance_commitment: transition.new_commitment,
        }),
    )
}

pub async fn view_balance_handler(
    State(state): State<AppState>,
    Path(user_id): Path<u128>,
) -> (StatusCode, Json<ViewBalanceResponse>) {
    let commitment = state
        .user_cache
        .lock()
        .await
        .get_balance_commitment(user_id)
        .map(str::to_owned);
    (
        StatusCode::OK,
        Json(ViewBalanceResponse {
            balance_commitment: commitment,
        }),
    )
}

pub async fn open_position_handler(
    State(state): State<AppState>,
    Json(payload): Json<OpenPositionRequest>,
) -> (StatusCode, Json<OpenPositionResponse>) {
    let transition = match extract_balance_transition(&payload.balance_proof, payload.initial_margin) {
        Ok(transition) => transition,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OpenPositionResponse {
                    message,
                    position_id: 0,
                    balance_commitment: String::new(),
                    liquidation_price: 0,
                }),
            )
        }
    };

    let mut user_cache = state.user_cache.lock().await;
    let stored = match user_cache.get_balance_commitment(payload.user_id).map(str::to_owned) {
        Some(commitment) => commitment,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OpenPositionResponse {
                    message: "user has no initialized balance commitment".to_string(),
                    position_id: 0,
                    balance_commitment: String::new(),
                    liquidation_price: 0,
                }),
            )
        }
    };

    if stored != transition.old_commitment {
        return (
            StatusCode::BAD_REQUEST,
            Json(OpenPositionResponse {
                message: "old_commitment does not match stored user commitment".to_string(),
                position_id: 0,
                balance_commitment: String::new(),
                liquidation_price: 0,
            }),
        );
    }

    let opening_fee = (payload.notional as f64 * 0.01).ceil() as u64;
    let effective_margin = payload.initial_margin.saturating_sub(opening_fee);
    let margin_fraction = effective_margin as f64 / payload.notional as f64;
    let liquidation_price = if payload.direction {
        (payload.entry_price as f64 * (1.0 - margin_fraction)) as u64
    } else {
        (payload.entry_price as f64 * (1.0 + margin_fraction)) as u64
    };

    let position_id = state.position_cache.lock().await.n;
    let position = Position {
        id: position_id,
        direction: payload.direction,
        notional: payload.notional,
        entry_price: payload.entry_price,
        leverage: payload.leverage,
        initial_margin: payload.initial_margin,
        liquidation_price,
    };

    user_cache.set_balance_commitment(payload.user_id, transition.new_commitment.clone());
    user_cache.add_position(payload.user_id, position.clone());
    drop(user_cache);
    state.position_cache.lock().await.add_position(position);

    (
        StatusCode::OK,
        Json(OpenPositionResponse {
            message: "Position opened successfully".to_string(),
            position_id,
            balance_commitment: transition.new_commitment,
            liquidation_price,
        }),
    )
}
