use axum::{
    routing::{get, post}, Router,
};
use clap::{Parser, ValueEnum};
mod fhe;
mod liqudation;
use crate::liqudation::users::{create_user_handler, get_user_handler, deposit_handler, view_balance_handler, open_position_handler};
use crate::liqudation::cache::{AccountCache, CiphertextCache, PositionCache};
use std::sync::Arc;
use tokio::sync::Mutex;
use tfhe::{ClientKey};
use crate::liqudation::handlers::{encrypt_handler, get_ciphertext_handler, health_check_long_handler, funding_rate_long_pay_short_handler};
use crate::fhe::key_gen::{Backend, ServerKeyType, generate_and_save_keys_with_backend};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Choose the FHE backend
    #[arg(short, long, value_enum, default_value_t = BackendChoice::Cpu)]
    backend: BackendChoice,
    
    /// Server port
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum BackendChoice {
    /// Use CPU backend for FHE operations
    Cpu,
    /// Use ROCm GPU backend for FHE operations  
    Gpu,
}

impl From<BackendChoice> for Backend {
    fn from(choice: BackendChoice) -> Self {
        match choice {
            BackendChoice::Cpu => Backend::Cpu,
            BackendChoice::Gpu => Backend::Gpu,
        }
    }
}

#[derive(Clone)]
struct AppState {
    user_cache: Arc<Mutex<AccountCache>>,
    ciphertext_cache: Arc<Mutex<CiphertextCache>>,
    position_cache: Arc<Mutex<PositionCache>>,
    server_key: Arc<ServerKeyType>,
    client_key: Arc<ClientKey>,
}

pub trait KeyAccess {
    fn get_server_key(&self) -> Arc<ServerKeyType>;
    fn get_client_key(&self) -> Arc<ClientKey>;
}

impl KeyAccess for AppState {
    fn get_server_key(&self) -> Arc<ServerKeyType> {
        self.server_key.clone()
    }
    fn get_client_key(&self) -> Arc<ClientKey> {
        self.client_key.clone()
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("Starting Dark Perps server with {:?} backend...", args.backend);
    
    let backend = Backend::from(args.backend);
    let keys = match generate_and_save_keys_with_backend(backend) {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("Failed to generate/load keys: {}", e);
            return;
        }
    };

    // Set the global server key for FHE operations
    keys.server_key.set_as_global();
    
    let user_cache = Arc::new(Mutex::new(AccountCache::new()));
    let ciphertext_cache = Arc::new(Mutex::new(CiphertextCache::new()));
    let position_cache = Arc::new(Mutex::new(PositionCache::new()));
    
    let state = AppState { 
        user_cache: user_cache.clone(),
        ciphertext_cache: ciphertext_cache.clone(),
        position_cache: position_cache.clone(),
        server_key: Arc::new(keys.server_key),
        client_key: Arc::new(keys.client_key),
    };
    
    let app = Router::new()
        .route("/create_user", post(create_user_handler))
        .route("/get_user/:user_id", get(get_user_handler))
        .route("/encrypt", post(encrypt_handler))
        .route("/deposit", post(deposit_handler))
        .route("/view_balance/:user_id", get(view_balance_handler))
        .route("/get_ciphertext/:ciphertext_key", get(get_ciphertext_handler))
        .route("/open_position", post(open_position_handler))
        .route("/health_check_long", post(health_check_long_handler))
        .route("/funding_rate_long_pay_short", post(funding_rate_long_pay_short_handler))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", args.port);
    println!("Server running on http://{} with {:?} backend", addr, args.backend);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


