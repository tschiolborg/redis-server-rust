use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;

pub type SharedInfo = Arc<Info>;

#[derive(PartialEq, Clone, Copy)]
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

pub struct Replication {
    role: ReplicaRole,
    master_replid: Option<String>,
    master_repl_offset: Option<u64>,
    master_host: Option<String>,
    master_port: Option<u16>,
}

impl Replication {
    pub fn master_addr(&self) -> Option<String> {
        match (&self.master_host, self.master_port) {
            (Some(host), Some(port)) => Some(format!("{}:{}", host, port)),
            _ => None,
        }
    }
}

pub struct Info {
    pub replication: Replication,
}

impl Info {
    fn new(replication: Replication) -> Info {
        Info { replication }
    }

    pub fn get_section(&self, name: &str) -> Option<String> {
        let mut res = Vec::new();
        match name {
            "replication" => {
                res.push(format!("# {}\n", name));
                res.push(format!("role:{}\n", self.replication.role.to_string()));
                if let Some(master_replid) = &self.replication.master_replid {
                    res.push(format!("master_replid:{}\n", master_replid));
                }
                if let Some(master_repl_offset) = &self.replication.master_repl_offset {
                    res.push(format!("master_repl_offset:{}\n", master_repl_offset));
                }
                if let Some(master_host) = &self.replication.master_host {
                    res.push(format!("master_host:{}\n", master_host));
                }
                if let Some(master_port) = &self.replication.master_port {
                    res.push(format!("master_port:{}\n", master_port));
                }
                Some(res.join(""))
            }
            _ => None,
        }
    }

    pub fn get_all(&self) -> String {
        let mut res = Vec::new();

        let sections = vec!["replication"];

        for section in sections {
            if let Some(s) = self.get_section(section) {
                res.push(s);
            }
        }

        res.join("\n")
    }
}

pub fn create_info(
    role: ReplicaRole,
    master_host: Option<String>,
    master_port: Option<u16>,
) -> Info {
    let master_replid = match role {
        ReplicaRole::MASTER => Some(
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(40)
                .map(char::from)
                .collect(),
        ),
        ReplicaRole::SLAVE => None,
    };
    let master_repl_offset = match role {
        ReplicaRole::MASTER => Some(0),
        ReplicaRole::SLAVE => None,
    };

    Info::new(Replication {
        role,
        master_replid,
        master_repl_offset,
        master_host,
        master_port,
    })
}
