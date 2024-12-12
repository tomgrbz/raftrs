pub mod connection;
pub mod data;
pub mod state;
pub mod utils;

use std::collections::HashMap;

pub use connection::*;
pub use data::*;
use serde::{Deserialize, Serialize};
pub use state::*;
pub use utils::*;

pub type Ticks = u64;
pub const BROADCAST: &str = "FFFF";
pub type Term = usize;

pub enum Target {
    Single(String),
    BroadCast,
}

#[derive(Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
    state: HashMap<String, LogEntry>,
    pub committed_len: usize,
    pub applied_len: usize,
}

impl Log {
    pub fn new() -> Self {
        let new_state: HashMap<String, LogEntry> = HashMap::new();
        Self {
            entries: Vec::new(),
            state: new_state,
            committed_len: 0,
            applied_len: 0,
        }
    }

    pub fn get_last_idx(&self) -> usize {
        if self.entries.len() > 0 {
            self.entries.len() - 1
        } else {
            0
        }
    }

    pub fn get_last_term(&self) -> Term {
        self.entries.last().map(|entry| entry.term).unwrap_or(0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub term: usize,
    pub key: String,
    pub value: String,
}
