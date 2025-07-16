use axum::{
    routing::{get, post}, Router,
};
use serde::{Deserialize, Serialize};
mod fhe;
mod liqudation;
use crate::liqudation::users::{create_user_handler, get_user_handler};
use crate::liqudation::cache::AccountCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use tfhe::{ServerKey, ClientKey};


#[derive(Clone)]
struct AppState {
    cache: Arc<Mutex<AccountCache>>,
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

    let cache = Arc::new(Mutex::new(AccountCache::new()));
    let state = AppState { 
        cache: cache.clone(),
        server_key: Arc::new(fhe::key_gen::load_server_key().unwrap()),
        client_key: Arc::new(fhe::key_gen::load_client_key().unwrap()),
    };
    
    let app = Router::new()
        .route("/create_user", post(create_user_handler))
        .route("/get_user/:user_id", get(get_user_handler))
        .with_state(state);


    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


