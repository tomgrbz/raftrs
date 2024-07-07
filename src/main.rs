use clap::Parser;
use raft_rs::{
    ConnInfo, ConnectionGroup, FailMessage, GetMessage, Message, MessageType, PutMessage,
};
use raft_rs::{HelloMessage, Peer};

use anyhow::{anyhow, Result};

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

const BROADCAST: &str = "FFFF";

type Ticks = u64;

type Term = usize;

fn rand_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
}

fn rand_jitter() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(60..=150)
}

fn main() {
    let cli = Cli::parse();
    let conn_info = ConnInfo::new(cli.port);
    let conn_group = ConnectionGroup::new(conn_info);
    let mut rep = Replica::new(cli.port, cli.id, cli.others, conn_group).unwrap();

    rep.run();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    port: u16,
    id: String,
    others: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum StateRole {
    /// The node is a follower of the leader.
    Follower(FollowerState),
    /// The node could become a leader.
    Candidate(CandidateState),
    /// The node is a leader.
    Leader(LeaderState),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct LeaderState {
    followers: HashMap<Peer, VolatileState>,
    heartbeat: Ticks,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct CandidateState {
    election_time: Ticks,
    votes_recv: HashSet<Peer>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct FollowerState {
    election_time: Ticks,
    leader: Option<Peer>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct VolatileState {
    next_index: u64,
    match_index: u64,
}

struct Replica {
    conn: ConnectionGroup,
    port: u16,
    id: String,
    role: StateRole,
    others: Vec<Peer>,
    leader: Peer,
}

impl Replica {
    fn new(
        port: u16,
        id: String,
        others: Vec<impl Into<String>>,
        conn: ConnectionGroup,
    ) -> Result<Self> {
        let mut peers = Vec::new();
        for peer in others {
            peers.push(Peer::new(peer.into()));
        }

        let role = {
            if id == "0000" {
                let mut followers = HashMap::new();
                for peer in peers.iter() {
                    followers.insert(
                        peer.clone(),
                        VolatileState {
                            next_index: 1,
                            match_index: 0,
                        },
                    );
                }

                StateRole::Leader(LeaderState {
                    followers: followers,
                    heartbeat: rand_jitter(),
                })
            } else {
                StateRole::Follower(FollowerState {
                    election_time: rand_jitter(),
                    leader: Some(Peer::new("0000".into())),
                })
            }
        };

        let rep = Replica {
            port,
            id,
            role,
            leader: Peer::new("0000".into()),
            others: peers,
            conn: conn,
        };

        let msg = Message::Hello(HelloMessage {
            src: rep.id.clone(),
            dst: BROADCAST.into(),
            leader: BROADCAST.into(),
            mid: rand_string(),
        });

        let _ = rep.send(msg).unwrap();
        println!("Created Replica object and sent hello");
        Ok(rep)
    }

    fn send(&self, message: Message) -> Result<()> {
        match self.conn.send_message(message) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Failed to send msg")),
        }
    }
    fn tick(&mut self) {
        println!("ticking now from id: {}", self.id);
        if let StateRole::Leader(LeaderState { .. }) = self.role {
            self.tick_heartbeat();
        } else {
            self.tick_election();
        }
    }
    fn tick_heartbeat(&mut self) {
        todo!()

        //self.handle_message(&RecvMessage::AppendEntriesMessage)
    }

    fn tick_election(&mut self) {
        todo!()
    }

    fn run(&mut self) -> Result<()> {
        let mut time = Instant::now();
        loop {
            let recv = self.conn.capture_recv_messages();

            match recv {
                Ok(msg) => self.handle_message(&msg),
                Err(e) => return  Err(e),
            };
            match self.role {
                StateRole::Candidate(CandidateState { election_time, .. })
                | StateRole::Follower(FollowerState { election_time, .. }) => {
                    if time.elapsed() > Duration::from_millis(election_time) {
                        self.tick();
                        time = Instant::now();
                    }
                }
                StateRole::Leader(LeaderState { heartbeat, .. }) => {
                    if time.elapsed() > Duration::from_millis(heartbeat) {
                        self.tick();
                        time = Instant::now();
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_message(&self, msg: &Message) {
        println!("{msg}");
        match msg {
            Message::Get(GetMessage {
                src,
                dst,
                leader,
                mid,
                key,
            })
            | Message::Put(PutMessage {
                src,
                dst,
                leader,
                mid,
                key,
                ..
            }) => {
                let msg = Message::Fail(FailMessage {
                    src: self.id.clone(),
                    dst: src.into(),
                    leader: self.leader.get_peer_id(),
                    mid: mid.into(),
                });
                self.conn.send_message(msg);
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use raft_rs::{ConnInfo, ConnectionGroup};

    use crate::Replica;

    #[test]
    fn test_new_replica() {
        let conn_info = ConnInfo::new(9090);
        let conn = ConnectionGroup::new(conn_info);
        let r = Replica::new(9090, "id".into(), vec!["1209", "1231"], conn).unwrap();
    }
}
