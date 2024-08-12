use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Section = HashMap<String, String>;
type Info = HashMap<String, Section>;

pub type SharedInfo = Arc<RwLock<Info>>;

pub enum ReplicaRole {
    MASTER,
    SLAVE,
}

impl ReplicaRole {
    fn to_string(&self) -> String {
        match self {
            ReplicaRole::MASTER => "master".to_string(),
            ReplicaRole::SLAVE => "slave".to_string(),
        }
    }
}

pub fn create_info(role: ReplicaRole) -> Info {
    let mut info = Info::new();

    let mut replication = Section::new();
    replication.insert("role".to_string(), role.to_string());

    info.insert("replication".to_string(), replication);

    info
}

pub fn print_section(mut res: Vec<String>, section: &Section) {
    for (k, v) in section.iter() {
        res.push(format!("{}:{}", k, v));
    }
    res.push("\n".to_string());
}
