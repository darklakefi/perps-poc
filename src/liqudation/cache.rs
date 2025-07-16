use crate::liqudation::users::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AccountCache {
    users: HashMap<u128, User>,
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

    pub fn get_user(&mut self, user_id: u128) -> Option<&mut User> {
        self.users.get_mut(&user_id)
    }

    pub fn user_exists(&self, user_id: u128) -> bool {
        self.users.contains_key(&user_id)
    }

    pub fn get_all_users(&self) -> &HashMap<u128, User> {
        &self.users
    }
    
}

pub type SharedAccountCache = Arc<Mutex<AccountCache>>;

