use std::sync::{Arc, Mutex, RwLock};

use serde::Serialize;

#[derive(Serialize,Clone)]
pub struct Subscription{
    pub id: i32,
    pub username: String,
    pub email: String,

}

#[derive(Clone)]
pub struct AppState{
    pub subscriptions: Arc<RwLock<Vec<Subscription>>>,
    next_id: Arc<Mutex<i32>>,
}

impl AppState {
    pub fn new() -> Self {
        let max_id = 0;
        Self {
            next_id: Arc::new(Mutex::new(max_id + 1)),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub fn get_id(&self) -> i32 {
        let mut next_id = self.next_id.lock().expect("mutex poisoned");
        let id = *next_id;
        *next_id += 1;
        id
    }
   
}
