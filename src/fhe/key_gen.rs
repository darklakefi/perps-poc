use std::fs;
use std::path::Path;
use tfhe::{ClientKey, ServerKey, CompactPublicKey, CompressedServerKey};
use tfhe::shortint::prelude::PARAM_MESSAGE_2_CARRY_2;
use tfhe::shortint::parameters::COMP_PARAM_MESSAGE_2_CARRY_2;

const KEYS_DIR: &str = "keys";
const CLIENT_KEY_PATH: &str = "keys/client_key.bin";
const SERVER_KEY_PATH: &str = "keys/server_key.bin";
const PUBLIC_KEY_PATH: &str = "keys/public_key.bin";
const COMPRESSED_SERVER_KEY_PATH: &str = "keys/compressed_server_key.bin";

#[derive(Debug, Clone)]
pub enum Backend {
    Cpu,
    Gpu,
}

pub struct GeneratedKeys {
    pub client_key: ClientKey,
    pub server_key: ServerKeyType,
}

#[derive(Clone)]
pub enum ServerKeyType {
    Cpu(ServerKey),
    #[cfg(any(feature = "gpu", feature = "gpu-rocm"))]
    Gpu(tfhe::CudaServerKey),
}

impl ServerKeyType {
    pub fn set_as_global(&self) {
        match self {
            ServerKeyType::Cpu(sk) => tfhe::set_server_key(sk.clone()),
            #[cfg(any(feature = "gpu", feature = "gpu-rocm"))]
            ServerKeyType::Gpu(sk) => tfhe::set_server_key(sk.clone()),
        }
    }
}

pub fn generate_and_save_keys_with_backend(backend: Backend) -> Result<GeneratedKeys, Box<dyn std::error::Error>> {
    println!("Checking keys...");
    if !Path::new(KEYS_DIR).exists() {
        fs::create_dir(KEYS_DIR)?;
    }

    if Path::new(CLIENT_KEY_PATH).exists() && 
       Path::new(COMPRESSED_SERVER_KEY_PATH).exists() {
        println!("Keys already exist. Loading existing keys...");
        return load_keys_with_backend(backend);
    } else {
        println!("Generating new keys for {:?} backend...", backend);
        let config = tfhe::ConfigBuilder::with_custom_parameters(PARAM_MESSAGE_2_CARRY_2)
            .enable_compression(COMP_PARAM_MESSAGE_2_CARRY_2)
            .build();
        
        let client_key = tfhe::ClientKey::generate(config);
        let compressed_server_key = tfhe::CompressedServerKey::new(&client_key);
        let public_key = tfhe::CompactPublicKey::new(&client_key);
        
        // Save compressed server key for GPU decompression later
        save_client_key(&client_key)?;
        save_compressed_server_key(&compressed_server_key)?;
        save_public_key(&public_key)?;
        
        let server_key = match backend {
            Backend::Cpu => {
                println!("Creating CPU server key...");
                ServerKeyType::Cpu(compressed_server_key.decompress())
            },
            #[cfg(any(feature = "gpu", feature = "gpu-rocm"))]
            Backend::Gpu => {
                println!("Creating GPU server key (decompressing to ROCm GPU)...");
                ServerKeyType::Gpu(compressed_server_key.decompress_to_gpu())
            },
            #[cfg(not(any(feature = "gpu", feature = "gpu-rocm")))]
            Backend::Gpu => {
                return Err("GPU backend requested but gpu feature not enabled".into());
            }
        };
        
        println!("Keys generated successfully for {:?} backend.", backend);
        Ok(GeneratedKeys {
            client_key,
            server_key,
        })
    }
}

fn load_keys_with_backend(backend: Backend) -> Result<GeneratedKeys, Box<dyn std::error::Error>> {
    let client_key = load_client_key()?;
    let compressed_server_key = load_compressed_server_key()?;
    
    let server_key = match backend {
        Backend::Cpu => {
            println!("Loading CPU server key...");
            ServerKeyType::Cpu(compressed_server_key.decompress())
        },
        #[cfg(any(feature = "gpu", feature = "gpu-rocm"))]
        Backend::Gpu => {
            println!("Loading GPU server key (decompressing to ROCm GPU)...");
            ServerKeyType::Gpu(compressed_server_key.decompress_to_gpu())
        },
        #[cfg(not(any(feature = "gpu", feature = "gpu-rocm")))]
        Backend::Gpu => {
            return Err("GPU backend requested but gpu feature not enabled".into());
        }
    };
    
    Ok(GeneratedKeys {
        client_key,
        server_key,
    })
}

// Keep the original function for backward compatibility
pub fn generate_and_save_keys() -> Result<(), Box<dyn std::error::Error>> {
    generate_and_save_keys_with_backend(Backend::Cpu).map(|_| ())
}

fn save_client_key(key: &ClientKey) -> Result<(), String> {
    let buffer = bincode::serialize(key)
        .map_err(|e| format!("Failed to serialize client key: {}", e))?;
    fs::write(CLIENT_KEY_PATH, buffer)
        .map_err(|e| format!("Failed to save client key: {}", e))?;
    Ok(())
}

fn save_server_key(key: &ServerKey) -> Result<(), String> {
    let buffer = bincode::serialize(key)
        .map_err(|e| format!("Failed to serialize server key: {}", e))?;
    fs::write(SERVER_KEY_PATH, buffer)
        .map_err(|e| format!("Failed to save server key: {}", e))?;
    Ok(())
}

fn save_public_key(key: &CompactPublicKey) -> Result<(), String> {
    let buffer = bincode::serialize(key)
        .map_err(|e| format!("Failed to serialize public key: {}", e))?;
    fs::write(PUBLIC_KEY_PATH, buffer)
        .map_err(|e| format!("Failed to save public key: {}", e))?;
    Ok(())
}

pub fn load_client_key() -> Result<ClientKey, String> {
    let data = fs::read(CLIENT_KEY_PATH)
        .map_err(|e| format!("Failed to read client key: {}", e))?;
    
    bincode::deserialize(&data)
        .map_err(|e| format!("Failed to deserialize client key: {}", e))
}

pub fn load_server_key() -> Result<ServerKey, String> {
    let data = fs::read(SERVER_KEY_PATH)
        .map_err(|e| format!("Failed to read server key: {}", e))?;
    bincode::deserialize(&data).map_err(|e| e.to_string())
}

pub fn load_public_key() -> Result<CompactPublicKey, String> {
    let data = fs::read(PUBLIC_KEY_PATH)
        .map_err(|e| format!("Failed to read public key: {}", e))?;
    bincode::deserialize(&data).map_err(|e| format!("Failed to deserialize public key: {}", e))
}

fn save_compressed_server_key(key: &CompressedServerKey) -> Result<(), String> {
    let buffer = bincode::serialize(key)
        .map_err(|e| format!("Failed to serialize compressed server key: {}", e))?;
    fs::write(COMPRESSED_SERVER_KEY_PATH, buffer)
        .map_err(|e| format!("Failed to save compressed server key: {}", e))?;
    Ok(())
}

fn load_compressed_server_key() -> Result<CompressedServerKey, String> {
    let data = fs::read(COMPRESSED_SERVER_KEY_PATH)
        .map_err(|e| format!("Failed to read compressed server key: {}", e))?;
    bincode::deserialize(&data)
        .map_err(|e| format!("Failed to deserialize compressed server key: {}", e))
}

