use axum::{
    routing::{get, post}, Router, Json, extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
mod fhe;
mod liqudation;
use crate::liqudation::users::{create_user_handler, get_user_handler};
use crate::liqudation::cache::{AccountCache, SharedAccountCache};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    if let Err(e) = fhe::key_gen::generate_and_save_keys() {
        eprintln!("Failed to generate keys: {}", e);
        return;
    }

    // Initialize the account cache as shared state
    let cache = Arc::new(Mutex::new(AccountCache::new()));
    
    let app = Router::new()
        .route("/create_user", post(create_user_handler))
        .route("/get_user", get(get_user_handler))
        .with_state(cache);


    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


