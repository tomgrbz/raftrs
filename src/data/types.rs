use std::fmt::{Display, Formatter, Pointer};
use serde::{Serialize, Serializer};

pub struct Peer {
    peer_id: String,
}
impl Peer {
    pub fn new(id: String) -> Self {
        Peer {
            peer_id: id
        }
    }

    fn get_peer_id(&self) -> String {
        return self.peer_id.clone()
    }
}

impl Serialize for Peer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.get_peer_id())
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_peer_id())
    }
}
