use crate::liqudation::users::{User, Position};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tfhe::{CompressedCiphertextList, FheUint64};



#[derive(Clone)]
pub struct Ciphertext {
    pub key: [u8;32],
    pub ciphertext: FheUint64,
    pub owner: u128,
}

#[derive(Clone)]
pub struct AccountCache {
    users: HashMap<u128, User>,
}

#[derive(Clone)]
pub struct PositionCache {
    pub n: u128,
    long_positions: Vec<Position>,
    short_positions: Vec<Position>,
}

impl AccountCache {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> bool {
        if self.users.contains_key(&user.id) {
            false // User already exists
        } else {
            self.users.insert(user.id, user);
            true // User added successfully
        }
    }

    pub fn update_balance(&mut self, user_id: u128, key: [u8;32]) {
        self.users.get_mut(&user_id).unwrap().balance = key;
    }

    pub fn get_user(&mut self, user_id: u128) -> Option<&mut User> {
        self.users.get_mut(&user_id)
    }

    pub fn get_balance(&self, user_id: u128) -> Option<&[u8;32]> {
        self.users.get(&user_id).map(|user| &user.balance)
    }

    pub fn user_exists(&self, user_id: u128) -> bool {
        self.users.contains_key(&user_id)
    }

    pub fn get_all_users(&self) -> &HashMap<u128, User> {
        &self.users
    }

    pub fn add_position(&mut self, user_id: u128, position: Position) {
        self.users.get_mut(&user_id).unwrap().positions.push(position);
    }
    
}

pub type SharedAccountCache = Arc<Mutex<AccountCache>>;

#[derive(Clone)]
pub struct CiphertextCache {
    ciphertexts: HashMap<[u8;32], Ciphertext>,
}

impl CiphertextCache {
    pub fn new() -> Self {
        Self {
            ciphertexts: HashMap::new(),
        }
    }
    
    pub fn add_ciphertext(&mut self, key: [u8;32], owner: u128, value: FheUint64) -> bool {
        if self.ciphertexts.contains_key(&key) {
            false // Ciphertext already exists
        } else {
            self.ciphertexts.insert(key, Ciphertext { key, owner, ciphertext: value });
            true // Ciphertext added successfully
        }
    }

    pub fn update_ciphertext(&mut self, key: [u8;32], owner: u128, value: FheUint64) -> bool {
        if !self.ciphertexts.contains_key(&key) {
            println!("update attempt failed: Ciphertext does not exist");
            false // Ciphertext does not exist
        } else {
            self.ciphertexts.insert(key, Ciphertext { key, owner, ciphertext: value });   
            println!("update attempt successful: Ciphertext updated");
            true // Ciphertext updated successfully
        }
    }

    pub fn get_ciphertext(&self, key: [u8;32]) -> Option<&Ciphertext> {
        self.ciphertexts.get(&key)
    }
       
}

impl PositionCache {
    pub fn new() -> Self {
        Self {
            n: 0,
            long_positions: Vec::new(),
            short_positions: Vec::new(),
        }
    }

    pub fn add_position(&mut self, position: Position) {
        self.n += 1;
        if position.direction {
            self.long_positions.push(position);
        } else {
            self.short_positions.push(position);
        }
    }

    pub fn get_position(&self, id: u128, direction: bool) -> Option<&Position> {
        if direction {
            self.long_positions.iter().find(|position| position.id == id)
        } else {
            self.short_positions.iter().find(|position| position.id == id)
        }
    }

}