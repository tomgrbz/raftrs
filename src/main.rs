use clap::Parser;
use raft_rs::{
    AppendEntriesMessage, AppendEntriesMessageResponse, ConnInfo, ConnectionGroup, FailMessage, GetMessage, Log, Message, MessageType, PutMessage, RequestVoteMessage, RequestVoteResponseMessage, BROADCAST
};
use raft_rs::{HelloMessage, Peer, Term, Ticks};

use anyhow::{anyhow, Result};
use tokio::time;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::vec;

fn rand_string() -> String {
    use rand::distributions::{Alphanumeric, DistString};
    Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
}

fn rand_jitter() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(150..=300)
}

fn rand_heartbeat_inteval() -> Ticks {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();
    rng.gen_range(50..=90)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let conn_info = ConnInfo::new(cli.port)
        .await
        .expect("Failed to bind to socket");
    let conn_group = ConnectionGroup::new(conn_info);
    let mut rep = Replica::new(cli.port, cli.id, cli.others, conn_group)
        .await
        .unwrap();

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
    pub next_index: usize,
    pub match_index: usize,
}

struct Replica {
    conn: ConnectionGroup,
    port: u16,
    id: String,
    role: StateRole,
    others: Vec<Peer>,
    voted_for: Option<Peer>,
    pub log: Log,
    pub term: Term,
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

        // let role = {
        //     if id == "0000" {
        //         let mut followers = HashMap::new();
        //         for peer in peers.iter() {
        //             followers.insert(
        //                 peer.clone(),
        //                 VolatileState {
        //                     next_index: 1,
        //                     match_index: 0,
        //                 },
        //             );
        //         }

        //         StateRole::Leader(LeaderState {
        //             followers: followers,
        //             heartbeat: rand_heartbeat_inteval(),
        //         })
        //     } else {
        //         StateRole::Follower(FollowerState {
        //             election_time: rand_jitter(),
        //             leader: None,
        //         })
        //     }
        // };
        let role = {
            StateRole::Follower(FollowerState {
                            election_time: rand_jitter(),
                            leader: None,
                        })
        };
        let rep = Replica {
            port,
            id,
            role,
            others: peers,
            conn: conn,
            voted_for: None,
            log: Log::new(),
            term: 0,
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

    fn quorum_size(&self) -> usize {
        self.others.len() / 2 + 1
    }

    async fn send(&self, message: Message) -> Result<()> {
        match self.conn.send_message(message).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to send msg, with {e}")),
        }
    }
    async fn tick(&mut self) {
        if let StateRole::Leader(LeaderState { .. }) = self.role {
            self.tick_heartbeat().await;
        } else {
            let _ = self.tick_election().await;
        }
    }

    async fn tick_heartbeat(&mut self) {
        self.replicate_log().await;
    }

    async fn replicate_log(&mut self) {
        let mut messages_to_send = Vec::new();
        if let StateRole::Leader(LeaderState {
            followers,
            heartbeat,
        }) = &mut self.role
        {
            *heartbeat = rand_heartbeat_inteval();

            for (follower, state) in followers.iter() {
                let prev_idx = state.next_index;

                let act = AppendEntriesMessage {
                    src: self.id.clone(),
                    dst: follower.get_peer_id(),
                    leader: self.id.clone(),
                    mid: rand_string(),
                    term: self.term,
                    prev_log_index: prev_idx,
                    prev_log_term: self.log.entries.get(prev_idx-1).unwrap().term,
                    entries: Vec::new(),
                    leader_commit_idx: self.log.committed_len,
                };
                let message = Message::AppendEntries(act);
                messages_to_send.push(message);
            }
        }
        futures::future::join_all(messages_to_send.into_iter().map(|msg| self.send(msg))).await;
    }

    async fn process_append_entries(&mut self, msg: AppendEntriesMessage) {
        if msg.term > self.term {
            self.reset_to_follower(msg.term);
        }

        match &mut self.role {
            StateRole::Candidate(_) | StateRole::Leader(_) => {
                if msg.term == self.term {
                    self.reset_to_follower(msg.term);
                    // self.process_append_entries(msg).await;
                }
            },
            StateRole::Follower(state) => {
                if msg.term == self.term {
                    state.leader = Some(Peer::new(msg.leader));
                    state.election_time = rand_jitter();

                    let prefix_len = msg.prev_log_index;

                    let prefix_ok = self.log.entries.len() >= prefix_len;
                    let last_terms_match = prefix_len == 0  || 
                        self.log.entries.get(prefix_len-1).expect("Failed to get valid keyentry").term == msg.term;
                    if prefix_ok && last_terms_match {
                        let msg = AppendEntriesMessageResponse {
                            src: self.id.clone(),
                            dst: msg.src.clone(),
                            leader: state.leader.clone().unwrap().get_peer_id(),
                            mid: msg.mid,
                            term: self.term,
                            success: true,
                        };
                        let _ = self.send(Message::AppendEntriesResponse(msg)).await;
                        true
                    } else {
                        false
                    }
                    
                }
                
            }
        }

    }

    async fn tick_election(&mut self) -> Result<()> {
        if let StateRole::Follower(FollowerState { election_time, .. })
        | StateRole::Candidate(CandidateState { election_time, .. }) = &mut self.role
        {
            let mut my_votes = HashSet::new();
            my_votes.insert(Peer::new(self.id.clone()));
            self.role = StateRole::Candidate(CandidateState {
                election_time: rand_jitter(),
                votes_recv: my_votes,
            });
            self.term += 1;
            self.voted_for = Some(Peer::new(self.id.clone()));

            let msg = RequestVoteMessage {
                src: self.id.clone(),
                dst: BROADCAST.to_string(),
                leader: BROADCAST.to_string(),
                mid: rand_string(),
                candidate_term: self.term,
                last_log_idx: self.log.get_last_idx(),
                last_log_term: self.log.get_last_term(),
            };
            self.send(Message::RequestVote(msg)).await?;
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        let mut time = Instant::now();
        loop {
            let recv = self.conn.capture_recv_messages().await;

            match recv {
                Ok(msg) => {
                    if let Message::AppendEntries(..) = msg {
                        println!("Received append rpc");
                        time = Instant::now();
                    }
                    self.handle_message(msg).await;
                }
                Err(e) => eprintln!("Failed to recv msg {e}"),
            };
            match self.role {
                StateRole::Candidate(CandidateState { election_time, .. })
                | StateRole::Follower(FollowerState { election_time, .. }) => {
                    if time.elapsed() > Duration::from_millis(election_time) {
                        self.tick().await;
                        time = Instant::now();
                    }
                }
                StateRole::Leader(LeaderState { heartbeat, .. }) => {
                    if time.elapsed() > Duration::from_millis(heartbeat) {
                        println!("leader sending heartbeat");
                        self.tick().await;
                        time = Instant::now();
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) {
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
            }) => {
                let msg = Message::Fail(FailMessage {
                    src: self.id.clone(),
                    dst: src.into(),
                    leader: {
                        if let StateRole::Follower(state) = self.role.clone() {
                            if let Some(leader) = state.leader {
                                leader.get_peer_id()
                            } else {
                                BROADCAST.into()
                            }
                        } else if let StateRole::Leader(_) = self.role.clone() {
                            self.id.clone()
                        } else {
                            BROADCAST.into()
                        }
                    },
                    mid: mid.into(),
                });
                let _ = self.conn.send_message(msg).await;
            }
            Message::RequestVote(msg) => {
                self.process_election_request(msg).await;
            },
            Message::RequestVoteResponse(msg) => {
                self.process_election_response(msg).await;
            }
            Message::AppendEntries(msg) => {
                self.process_append_entries(msg).await;
            }

            _ => {}
        }
    }

    fn process_append_entries_resp(&mut self, msg: AppendEntriesMessageResponse) {}

    // Processes a RPC for request vote from a candidate
    async fn process_election_request(&mut self, msg: RequestVoteMessage) {
        if msg.candidate_term > self.term {
            self.reset_to_follower(msg.candidate_term);
        }
        let havent_voted = match self.voted_for {
            Some(ref peer) => peer.get_peer_id() == msg.src,
            None => true,
        };

        // candidate term is as long or equal to this term, making it valid
        let cand_log_term_valid = msg.last_log_term > self.log.get_last_term();
        let cand_log_is_longer = msg.last_log_term == self.log.get_last_term()
            && msg.last_log_idx > self.log.get_last_idx();
        let log_is_valid = cand_log_term_valid || cand_log_is_longer;

        let up_to_date = msg.candidate_term == self.term;

        let success = if up_to_date && log_is_valid && havent_voted {
            self.voted_for = Some(Peer::new(msg.src.clone()));
            true
        } else {
            println!("Failed to vote for rep: {}", msg.src);
            false
        };

        let vote_response = RequestVoteResponseMessage {
            src: self.id.clone(),
            dst: msg.src.clone(),
            leader: BROADCAST.into(),
            mid: msg.mid,
            term: self.term,
            voted_granted: success,
        };

        let _ = self.send(Message::RequestVoteResponse(vote_response)).await;
    }

    async fn process_election_response(&mut self, msg: RequestVoteResponseMessage) {
        if msg.term > self.term {
            self.reset_to_follower(msg.term);
        }

        if let StateRole::Candidate(state) = &mut self.role {
            let same_term = msg.term == self.term;

            if same_term && msg.voted_granted {
                state.votes_recv.insert(Peer::new(msg.src));

                if state.votes_recv.len() > self.quorum_size() {
                    let mut follower_list_as_leader = HashMap::new();
                    self.others.iter().for_each(|peer| {
                        match follower_list_as_leader.insert(
                            peer.clone(),
                            VolatileState {
                                next_index: self.log.get_last_idx(),
                                match_index: 0,
                            },
                        ) {
                            None => {
                                println!("Invalid insert for new election");
                            }
                            _ => {}
                        }
                    });
                    self.promote_to_leader(follower_list_as_leader).await;
                }
            }
        }
    }

    fn reset_to_follower(&mut self, term: Term) {
        if term > self.term {
            self.term = term;
            self.role = StateRole::Follower(FollowerState {
                election_time: rand_jitter(),
                leader: None,
            });
        }
        self.voted_for = None;
    }

    async fn promote_to_leader(&mut self, followers: HashMap<Peer, VolatileState>) {
        self.role = StateRole::Leader(LeaderState {
            followers: followers,
            heartbeat: rand_heartbeat_inteval(),
        });
        println!("NEW LEADER ELECTED: {}", self.id.clone());
        self.replicate_log().await;
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
        let r = Replica::new(9090, "id".into(), vec!["1209", "1231"], conn)
            .await
            .unwrap();
    }
}
