use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub type SharedData = Arc<RwLock<dyn Data + Send + Sync>>;

pub struct Item {
    value: String,
    created_at: Instant,
    px: Option<u128>,
}

pub trait Data {
    fn get(&self, key: &str) -> Option<String>;

    fn set(&mut self, key: String, value: String, px: Option<u128>);
}

pub struct InMemoryData {
    data: HashMap<String, Item>,
}

impl InMemoryData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Data for InMemoryData {
    fn get(&self, key: &str) -> Option<String> {
        let item = self.data.get(key)?;

        if item.px.is_some() && item.created_at.elapsed().as_millis() > item.px? {
            None
        } else {
            Some(item.value.clone())
        }
    }

    fn set(&mut self, key: String, value: String, px: Option<u128>) {
        self.data.insert(
            key,
            Item {
                value,
                created_at: Instant::now(),
                px,
            },
        );
    }
}
