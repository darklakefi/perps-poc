use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct HealthCheckRequest {
    pub position_id: u128,
    pub mark_price: u64,
}

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub liquidation_price: u64,
}

#[derive(Deserialize)]
pub struct FundingRateLPSRequest {
    pub position_id: u128,
    pub delta_percent: u64,
}

#[derive(Serialize)]
pub struct FundingRateLPSResponse {
    pub status: String,
    pub liquidation_price: u64,
}

pub async fn health_check_long_handler(
    State(state): State<AppState>,
    Json(payload): Json<HealthCheckRequest>,
) -> (StatusCode, Json<HealthCheckResponse>) {
    let position = match state.position_cache.lock().await.get_position(payload.position_id, true).cloned() {
        Some(position) => position,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(HealthCheckResponse {
                    status: "Position not found".to_string(),
                    liquidation_price: 0,
                }),
            )
        }
    };

    let solvent = payload.mark_price >= position.liquidation_price;
    (
        if solvent { StatusCode::OK } else { StatusCode::BAD_REQUEST },
        Json(HealthCheckResponse {
            status: if solvent {
                "Solvent".to_string()
            } else {
                "Insolvent".to_string()
            },
            liquidation_price: position.liquidation_price,
        }),
    )
}

pub async fn funding_rate_long_pay_short_handler(
    State(state): State<AppState>,
    Json(payload): Json<FundingRateLPSRequest>,
) -> (StatusCode, Json<FundingRateLPSResponse>) {
    let mut position_cache = state.position_cache.lock().await;
    let position = match position_cache.get_any_position_mut(payload.position_id) {
        Some(position) => position,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(FundingRateLPSResponse {
                    status: "Position not found".to_string(),
                    liquidation_price: 0,
                }),
            )
        }
    };

    let delta = position.notional.saturating_mul(payload.delta_percent) / 100;
    if position.direction {
        position.liquidation_price = position.liquidation_price.saturating_sub(delta);
    } else {
        position.liquidation_price = position.liquidation_price.saturating_add(delta);
    }

    (
        StatusCode::OK,
        Json(FundingRateLPSResponse {
            status: "Success".to_string(),
            liquidation_price: position.liquidation_price,
        }),
    )
}
