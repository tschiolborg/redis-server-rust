use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedData = Arc<RwLock<dyn Data + Send + Sync>>;

pub trait Data {
    fn get(&self, key: &str) -> Option<String>;

    fn set(&mut self, key: String, value: String);
}

pub struct InMemoryData {
    data: std::collections::HashMap<String, String>,
}

impl InMemoryData {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl Data for InMemoryData {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
}
