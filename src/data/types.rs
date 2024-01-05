use serde::{Serialize, Serializer};

pub struct Peer {
    pub peer_id: String,
}

impl Serialize for Peer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.collect_str(&self.peer_id)
    }
}