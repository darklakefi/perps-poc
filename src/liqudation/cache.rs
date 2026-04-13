use crate::liqudation::users::{Position, User};
use std::collections::HashMap;

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
            false
        } else {
            self.users.insert(user.id, user);
            true
        }
    }

    pub fn get_user(&self, user_id: u128) -> Option<&User> {
        self.users.get(&user_id)
    }

    pub fn get_user_mut(&mut self, user_id: u128) -> Option<&mut User> {
        self.users.get_mut(&user_id)
    }

    pub fn get_balance_commitment(&self, user_id: u128) -> Option<&str> {
        self.users
            .get(&user_id)
            .and_then(|user| user.balance_commitment.as_deref())
    }

    pub fn set_balance_commitment(&mut self, user_id: u128, commitment: String) {
        self.users.get_mut(&user_id).unwrap().balance_commitment = Some(commitment);
    }

    pub fn add_position(&mut self, user_id: u128, position: Position) {
        self.users.get_mut(&user_id).unwrap().positions.push(position);
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

    pub fn get_any_position(&self, id: u128) -> Option<&Position> {
        self.long_positions
            .iter()
            .find(|position| position.id == id)
            .or_else(|| self.short_positions.iter().find(|position| position.id == id))
    }

    pub fn get_any_position_mut(&mut self, id: u128) -> Option<&mut Position> {
        if let Some(position) = self.long_positions.iter_mut().find(|position| position.id == id) {
            Some(position)
        } else {
            self.short_positions.iter_mut().find(|position| position.id == id)
        }
    }
}
