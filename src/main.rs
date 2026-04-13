use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

mod liqudation;
mod zyga_proof;

use crate::liqudation::cache::{AccountCache, PositionCache};
use crate::liqudation::handlers::{funding_rate_long_pay_short_handler, health_check_long_handler};
use crate::liqudation::users::{
    create_user_handler, deposit_handler, get_user_handler, open_position_handler,
    view_balance_handler,
};
use crate::zyga_proof::handlers::{health_check_zyga_handler, open_zyga_position_handler};

#[derive(Clone)]
struct AppState {
    user_cache: Arc<Mutex<AccountCache>>,
    position_cache: Arc<Mutex<PositionCache>>,
}

#[tokio::main]
async fn main() {
    let user_cache = Arc::new(Mutex::new(AccountCache::new()));
    let position_cache = Arc::new(Mutex::new(PositionCache::new()));
    let state = AppState {
        user_cache,
        position_cache,
    };

    let app = Router::new()
        .route("/create_user", post(create_user_handler))
        .route("/get_user/:user_id", get(get_user_handler))
        .route("/deposit", post(deposit_handler))
        .route("/view_balance/:user_id", get(view_balance_handler))
        .route("/open_position", post(open_position_handler))
        .route("/health_check_long", post(health_check_long_handler))
        .route("/funding_rate_long_pay_short", post(funding_rate_long_pay_short_handler))
        .route("/open_zyga_position", post(open_zyga_position_handler))
        .route("/health_check_zyga", post(health_check_zyga_handler))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
