use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub type SharedData = Arc<RwLock<dyn Data + Send + Sync>>;

#[derive(Clone)]
pub struct DataItem {
    value: String,
    created_at: Instant,
    px: Option<u128>,
}

impl DataItem {
    pub fn is_expired(&self) -> bool {
        self.px.is_some() && self.created_at.elapsed().as_millis() > self.px.unwrap()
    }
}

pub trait Data {
    fn get(&self, key: &str) -> Option<String>;

    fn set(&mut self, key: String, value: String, px: Option<u128>);

    fn del(&mut self, key: &str);

    fn expire_keys(&mut self);
}

pub struct InMemoryData {
    data: HashMap<String, DataItem>,
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

        if item.is_expired() {
            None
        } else {
            Some(item.value.clone())
        }
    }

    fn set(&mut self, key: String, value: String, px: Option<u128>) {
        self.data.insert(
            key,
            DataItem {
                value,
                created_at: Instant::now(),
                px,
            },
        );
    }

    fn del(&mut self, key: &str) {
        self.data.remove(key);
    }

    fn expire_keys(&mut self) {
        // can we do this without cloning?
        let keys = self
            .data
            .iter()
            .filter_map(|(key, item)| {
                if item.is_expired() {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        for key in keys {
            self.del(key.as_str());
        }
    }
}
