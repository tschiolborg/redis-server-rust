use std::sync::Arc;

// TODO: make struct
pub type SharedInfo = Arc<Info>;

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

struct Replication {
    role: ReplicaRole,
}

pub struct Info {
    replication: Replication,
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

pub fn create_info(role: ReplicaRole) -> Info {
    Info::new(Replication { role })
}
