use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::liqudation::users::Position;
use crate::zyga_proof::verifier::{
    calculate_liquidation_price_long, calculate_liquidation_price_short, extract_balance_transition,
    extract_mark_price_from_public_inputs, verify_liquidation_proof, VerificationResult,
};
use crate::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ZygaProofData {
    Simple {
        mark_price: u64,
    },
    Generated {
        mark_price: Option<u64>,
        public_inputs: Map<String, Value>,
        #[serde(default)]
        pairing_proof: Option<Value>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenZygaPositionRequest {
    pub user_id: u128,
    pub direction: bool,
    pub entry_price: u64,
    pub notional: u64,
    pub leverage: u64,
    pub initial_margin: u64,
    pub balance_proof: Value,
    pub liquidation_proof: ZygaProofData,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenZygaPositionResponse {
    pub success: bool,
    pub message: String,
    pub position_id: Option<u128>,
    pub server_liquidation_price: u64,
    pub proof_mark_price: u64,
    pub verification_passed: bool,
    pub verification_message: String,
    pub balance_commitment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckZygaRequest {
    pub position_id: u128,
    pub mark_price: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckZygaResponse {
    pub is_solvent: bool,
    pub message: String,
    pub mark_price: u64,
    pub liquidation_price: u64,
    pub direction: bool,
}

fn proof_mark_price(proof: &ZygaProofData) -> Result<u64, String> {
    match proof {
        ZygaProofData::Simple { mark_price } => Ok(*mark_price),
        ZygaProofData::Generated {
            mark_price,
            public_inputs,
            ..
        } => {
            let parsed = extract_mark_price_from_public_inputs(public_inputs)
                .map_err(|err| err.to_string())?;
            if let Some(explicit) = mark_price {
                if *explicit != parsed {
                    return Err(format!(
                        "mark_price mismatch: explicit {} != public_inputs {}",
                        explicit, parsed
                    ));
                }
            }
            Ok(parsed)
        }
    }
}

#[axum::debug_handler]
pub async fn open_zyga_position_handler(
    State(state): State<AppState>,
    Json(payload): Json<OpenZygaPositionRequest>,
) -> (StatusCode, Json<OpenZygaPositionResponse>) {
    let server_liquidation_price = if payload.direction {
        calculate_liquidation_price_long(
            payload.entry_price,
            payload.initial_margin,
            payload.notional,
        )
    } else {
        calculate_liquidation_price_short(
            payload.entry_price,
            payload.initial_margin,
            payload.notional,
        )
    };

    let proof_mark_price = match proof_mark_price(&payload.liquidation_proof) {
        Ok(mark_price) => mark_price,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OpenZygaPositionResponse {
                    success: false,
                    message: "Invalid liquidation proof payload".to_string(),
                    position_id: None,
                    server_liquidation_price,
                    proof_mark_price: 0,
                    verification_passed: false,
                    verification_message: message,
                    balance_commitment: None,
                }),
            );
        }
    };

    let transition = match extract_balance_transition(&payload.balance_proof, payload.initial_margin) {
        Ok(transition) => transition,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OpenZygaPositionResponse {
                    success: false,
                    message: "Invalid balance proof payload".to_string(),
                    position_id: None,
                    server_liquidation_price,
                    proof_mark_price,
                    verification_passed: false,
                    verification_message: message,
                    balance_commitment: None,
                }),
            );
        }
    };

    let VerificationResult {
        valid: verification_passed,
        message: verification_message,
    } = match verify_liquidation_proof(
        proof_mark_price,
        server_liquidation_price,
        payload.direction,
    ) {
        Ok(result) => result,
        Err(err) => VerificationResult {
            valid: false,
            message: err.to_string(),
        },
    };

    if !verification_passed {
        return (
            StatusCode::BAD_REQUEST,
            Json(OpenZygaPositionResponse {
                success: false,
                message: "Proof verification failed".to_string(),
                position_id: None,
                server_liquidation_price,
                proof_mark_price,
                verification_passed: false,
                verification_message,
                balance_commitment: None,
            }),
        );
    }

    let mut user_cache = state.user_cache.lock().await;
    let stored = match user_cache.get_balance_commitment(payload.user_id).map(str::to_owned) {
        Some(commitment) => commitment,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(OpenZygaPositionResponse {
                    success: false,
                    message: "user has no initialized balance commitment".to_string(),
                    position_id: None,
                    server_liquidation_price,
                    proof_mark_price,
                    verification_passed: true,
                    verification_message: "missing user balance commitment".to_string(),
                    balance_commitment: None,
                }),
            )
        }
    };

    if stored != transition.old_commitment {
        return (
            StatusCode::BAD_REQUEST,
            Json(OpenZygaPositionResponse {
                success: false,
                message: "old_commitment does not match stored user commitment".to_string(),
                position_id: None,
                server_liquidation_price,
                proof_mark_price,
                verification_passed: true,
                verification_message: "balance proof does not match account state".to_string(),
                balance_commitment: None,
            }),
        );
    }

    let position_id = state.position_cache.lock().await.n;
    let position = Position {
        id: position_id,
        direction: payload.direction,
        notional: payload.notional,
        entry_price: payload.entry_price,
        leverage: payload.leverage,
        initial_margin: payload.initial_margin,
        liquidation_price: server_liquidation_price,
    };

    user_cache.set_balance_commitment(payload.user_id, transition.new_commitment.clone());
    user_cache.add_position(payload.user_id, position.clone());
    drop(user_cache);
    state.position_cache.lock().await.add_position(position);

    (
        StatusCode::OK,
        Json(OpenZygaPositionResponse {
            success: true,
            message: "Position opened successfully with Zyga proofs".to_string(),
            position_id: Some(position_id),
            server_liquidation_price,
            proof_mark_price,
            verification_passed: true,
            verification_message,
            balance_commitment: Some(transition.new_commitment),
        }),
    )
}

#[axum::debug_handler]
pub async fn health_check_zyga_handler(
    State(state): State<AppState>,
    Json(payload): Json<HealthCheckZygaRequest>,
) -> (StatusCode, Json<HealthCheckZygaResponse>) {
    let position = match state.position_cache.lock().await.get_any_position(payload.position_id).cloned() {
        Some(position) => position,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(HealthCheckZygaResponse {
                    is_solvent: false,
                    message: "Position not found".to_string(),
                    mark_price: payload.mark_price,
                    liquidation_price: 0,
                    direction: true,
                }),
            );
        }
    };

    let is_solvent = if position.direction {
        payload.mark_price >= position.liquidation_price
    } else {
        payload.mark_price <= position.liquidation_price
    };

    (
        if is_solvent { StatusCode::OK } else { StatusCode::BAD_REQUEST },
        Json(HealthCheckZygaResponse {
            is_solvent,
            message: if is_solvent {
                format!("Position {} is solvent", payload.position_id)
            } else {
                format!("Position {} is at risk", payload.position_id)
            },
            mark_price: payload.mark_price,
            liquidation_price: position.liquidation_price,
            direction: position.direction,
        }),
    )
}
