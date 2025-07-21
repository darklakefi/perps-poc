use axum::{
    routing::{get, post}, Router, Json, extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
mod fhe;
mod liqudation;
use crate::liqudation::users::{create_user_handler, get_user_handler, deposit_handler, view_balance_handler, open_position_handler};
use crate::liqudation::cache::{AccountCache, SharedAccountCache, CiphertextCache, PositionCache};
use std::sync::Arc;
use tokio::sync::Mutex;
use tfhe::{ServerKey, ClientKey};
use crate::liqudation::handlers::{encrypt_handler, get_ciphertext_handler, health_check_long_handler};


#[derive(Clone)]
struct AppState {
    user_cache: Arc<Mutex<AccountCache>>,
    ciphertext_cache: Arc<Mutex<CiphertextCache>>,
    position_cache: Arc<Mutex<PositionCache>>,
    server_key: Arc<ServerKey>,
    client_key: Arc<ClientKey>,
}

pub trait KeyAccess {
    fn get_server_key(&self) -> Arc<ServerKey>;
    fn get_client_key(&self) -> Arc<ClientKey>;
}

impl KeyAccess for AppState {
    fn get_server_key(&self) -> Arc<ServerKey> {
        self.server_key.clone()
    }
    fn get_client_key(&self) -> Arc<ClientKey> {
        self.client_key.clone()
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = fhe::key_gen::generate_and_save_keys() {
        eprintln!("Failed to generate keys: {}", e);
        return;
    }

    let user_cache = Arc::new(Mutex::new(AccountCache::new()));
    let ciphertext_cache = Arc::new(Mutex::new(CiphertextCache::new()));
    let position_cache = Arc::new(Mutex::new(PositionCache::new()));
    let state = AppState { 
        user_cache: user_cache.clone(),
        ciphertext_cache: ciphertext_cache.clone(),
        position_cache: position_cache.clone(),
        server_key: Arc::new(fhe::key_gen::load_server_key().unwrap()),
        client_key: Arc::new(fhe::key_gen::load_client_key().unwrap()),
    };
    
    let app = Router::new()
        .route("/create_user", post(create_user_handler))
        .route("/get_user/:user_id", get(get_user_handler))
        .route("/encrypt", post(encrypt_handler))
        .route("/deposit", post(deposit_handler))
        .route("/view_balance/:user_id", get(view_balance_handler))
        .route("/get_ciphertext/:ciphertext_key", get(get_ciphertext_handler))
        .route("/open_position", post(open_position_handler)) // maybe i make a seperate one for long/short
        .route("/health_check_long", post(health_check_long_handler))
        .with_state(state);


    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


