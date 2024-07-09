use clap::Parser;
use raft_rs::{
    AppendEntriesMessage, ConnInfo, ConnectionGroup, FailMessage, GetMessage, Message, MessageType, PutMessage
};
use raft_rs::{HelloMessage, Peer};

use anyhow::{anyhow, Result};
use tokio::time;

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
    rng.gen_range(20..=80)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let conn_info = ConnInfo::new(cli.port).await.expect("Failed to bind to socket");
    let conn_group = ConnectionGroup::new(conn_info);
    let mut rep = Replica::new(cli.port, cli.id, cli.others, conn_group).await.unwrap();

    rep.run().await.unwrap();
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
    async fn new(
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

        let _ = rep.send(msg).await.unwrap();
        println!("Created Replica object and sent hello");
        Ok(rep)
    }

    async fn send(&self, message: Message) -> Result<()> {
        match self.conn.send_message(message).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to send msg, with {e}")),
        }
    }
    async fn tick(&mut self) {
        println!("ticking now from id: {}", self.id);
        if let StateRole::Leader(LeaderState { .. }) = self.role {
            self.tick_heartbeat().await;
        } else {
            self.tick_election().await;
        }
    }
    async fn tick_heartbeat(&mut self) {
        for peer in self.others.iter() {
            let act = AppendEntriesMessage {
                src: self.id.clone(),
                dst: peer.get_peer_id(),
                leader: self.id.clone(),
                mid: rand_string(),
            };
            let message = Message::AppendEntries(act);
            if let Err(e) = self.send(message).await {
                panic!("Failed to send to {peer}");
            }
        }

        //self.handle_message(&RecvMessage::AppendEntriesMessage)
    }

    async fn tick_election(&mut self) {
        todo!()
    }

    async fn run(&mut self) -> Result<()> {

        let mut time = Instant::now();
        loop {
            
            let recv = self.conn.capture_recv_messages().await;

            match recv {
                Ok(msg) => self.handle_message(&msg).await,
                Err(e) => eprintln!("Failed to recv msg {e}"),
            };
            match self.role {
                StateRole::Candidate(CandidateState { election_time, .. })
                | StateRole::Follower(FollowerState { election_time, .. }) => {
                    if time.elapsed() > Duration::from_millis(election_time) {
                        println!("leader sending heartbeat");
                        self.tick().await;
                        time = Instant::now();
                    }
                }
                StateRole::Leader(LeaderState { heartbeat, .. }) => {
                    if time.elapsed() > Duration::from_millis(heartbeat) {
                        self.tick().await;
                        time = Instant::now();
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_message(&self, msg: &Message) {
        println!("handling now {msg}");
        match msg {
            Message::Get(GetMessage {
                src,
                dst,
                leader,
                mid,
                ..
            })
            | Message::Put(PutMessage {
                src,
                dst,
                leader,
                mid,
                ..
            })
            | Message::AppendEntries(AppendEntriesMessage { 
                src, 
                dst, 
                leader, 
                mid }) => {
                let msg = Message::Fail(FailMessage {
                    src: self.id.clone(),
                    dst: src.into(),
                    leader: self.leader.get_peer_id(),
                    mid: mid.into(),
                });
                let _ = self.conn.send_message(msg).await;
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use raft_rs::{ConnInfo, ConnectionGroup};

    use crate::Replica;

    #[tokio::test]
    async fn test_new_replica() {
        let conn_info = ConnInfo::new(9090).await.unwrap();
        let conn = ConnectionGroup::new(conn_info);
        let r = Replica::new(9090, "id".into(), vec!["1209", "1231"], conn).await.unwrap();
    }
}
