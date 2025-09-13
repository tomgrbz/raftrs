use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use crate::{Peer, Ticks};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StateRole {
    /// The node is a follower of the leader.
    Follower(FollowerState),
    /// The node could become a leader.
    Candidate(CandidateState),
    /// The node is a leader.
    Leader(LeaderState),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LeaderState {
    pub followers: HashMap<Peer, VolatileState>,
    pub heartbeat_due: Instant,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CandidateState {
    pub votes_recv: HashSet<Peer>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FollowerState {
    pub leader: Option<Peer>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VolatileState {
    pub next_index: usize,
    pub match_index: usize,
}
