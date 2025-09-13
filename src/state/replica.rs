use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use super::StateRole;
use crate::{
    rand_heartbeat_inteval, rand_jitter, rand_string, AppendEntriesMessage,
    AppendEntriesMessageResponse, CandidateState, Connection, ConnectionGroup, FailMessage,
    FollowerState, GetMessage, HelloMessage, LeaderState, Log, LogEntry, Message, OkMessage, Peer,
    PutMessage, RedirectMessage, RequestVoteMessage, RequestVoteResponseMessage, Term, Ticks,
    VolatileState, BROADCAST,
};
use anyhow::{anyhow, Result};
use tokio::{signal, time::timeout};

#[derive(Debug)]
pub struct Replica<Conn: Connection> {
    conn: Conn,
    id: String,
    role: StateRole,
    others: Vec<Peer>,
    voted_for: Option<Peer>,
    pub log: Log,
    pub term: Term,
    pub ticks: Ticks,
    leader: String,
}

impl<Conn: Connection> Replica<Conn> {
    pub async fn new(id: String, others: Vec<impl Into<String>>, conn: Conn) -> Result<Self> {
        let mut peers = Vec::new();
        for peer in others {
            peers.push(Peer::new(peer.into()));
        }
        let rand_timeout = rand_jitter(None);
        let role = { StateRole::Follower(FollowerState { leader: None }) };
        let rep = Replica {
            id,
            role,
            others: peers,
            conn: conn,
            voted_for: None,
            log: Log::new(),
            term: 0,
            ticks: rand_timeout,
            leader: String::from("FFFF"),
        };

        Ok(rep)
    }

    fn quorum_size(&self) -> usize {
        self.others.len() / 2 + 1
    }

    async fn send(&mut self, message: Message) -> Result<()> {
        match self.conn.send_message(message).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to send msg, with {e}")),
        }
    }
    async fn tick(&mut self) {
        if let StateRole::Leader(LeaderState { .. }) = self.role {
            let _ = self.tick_heartbeat().await;
        } else {
            let _ = self.tick_election().await;
        }
    }

    async fn tick_heartbeat(&mut self) -> Result<()> {
        self.ticks = rand_heartbeat_inteval();
        self.replicate_log().await
    }

    async fn replicate_log(&mut self) -> Result<()> {
        let mut messages_to_send = Vec::new();
        if let StateRole::Leader(LeaderState { followers, .. }) = &mut self.role {
            // self.ticks = rand_heartbeat_inteval();

            for (follower, state) in followers.iter() {
                let prev_idx = if state.next_index == 0 {
                    0
                } else {
                    state.next_index - 1
                };
                let entry = if let Some(val) = self.log.entries.get(prev_idx) {
                    val.term
                } else {
                    0
                };
                let mid = rand_string();
                let follower_id = follower.get_peer_id();

                let act = AppendEntriesMessage {
                    src: self.id.clone(),
                    dst: follower_id,
                    leader: self.id.clone(),
                    mid: mid,
                    term: self.term,
                    prev_log_index: prev_idx,
                    prev_log_term: entry,
                    entries: Vec::new(),
                    leader_commit_idx: self.log.committed_len,
                };
                let message = Message::AppendEntries(act);
                messages_to_send.push(message);
            }
        }
        if let Err(e) = self.send_msgs(messages_to_send).await {
            if let StateRole::Leader(ref mut leader_state) = self.role {
                leader_state.heartbeat_due = Instant::now() + Duration::from_millis(150);
            }
            Err(anyhow::anyhow!("failed sending append entries RPC").context(e))
        } else {
            if let StateRole::Leader(ref mut leader_state) = self.role {
                leader_state.heartbeat_due = Instant::now() + Duration::from_millis(150);
            }
            Ok(())
        }
    }

    async fn send_msgs(&mut self, msgs: Vec<Message>) -> Result<()> {
        for m in msgs {
            self.send(m).await?;
        }
        Ok(())
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
            }
            StateRole::Follower(state) => {
                // if msg.term == self.term {

                self.ticks = rand_jitter(None);

                let prefix_len = msg.prev_log_index;

                let prefix_ok = self.log.entries.len() >= prefix_len;
                let last_terms_match = prefix_len == 0
                    || self
                        .log
                        .entries
                        .get(prefix_len - 1)
                        .expect("Failed to get valid keyentry")
                        .term
                        == msg.prev_log_term;
                if prefix_ok && last_terms_match {
                    state.leader = Some(Peer::new(msg.leader.clone()));
                    self.leader = msg.leader.clone();
                    let msg = AppendEntriesMessageResponse {
                        src: self.id.clone(),
                        dst: msg.src.clone(),
                        leader: state.leader.clone().unwrap().get_peer_id(),
                        mid: msg.mid,
                        term: self.term,
                        success: true,
                    };
                    let _ = self.send(Message::AppendEntriesResponse(msg)).await;
                }
            }
        }
    }

    async fn tick_election(&mut self) -> Result<()> {
        if let StateRole::Follower(_) | StateRole::Candidate(_) = &mut self.role {
            let mut my_votes = HashSet::new();
            my_votes.insert(Peer::new(self.id.clone()));
            self.role = StateRole::Candidate(CandidateState {
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

    pub async fn run(&mut self) -> Result<()> {
        let mid = rand_string();
        let msg = Message::Hello(HelloMessage {
            src: self.id.clone(),
            dst: BROADCAST.to_string(),
            leader: BROADCAST.to_string(),
            mid: mid,
        });

        if let Err(e) = self.send(msg).await {
            return Err(anyhow!("Failed to send first hello message").context(e));
        }
        loop {
            let recv = timeout(
                Duration::from_millis(self.ticks),
                self.conn.capture_recv_messages(),
            )
            .await;
            // if let Ok(()) = signal::ctrl_c().await {
            //     return Err(anyhow!("Program halted with CTRL > C"));
            // }

            match recv {
                Ok(inner_result) => {
                    if let Ok(msg) = inner_result {
                        if let (Message::RequestVote(v)) = &msg {
                            println!("Msg received  {:?}", v);
                        }

                        self.handle_message(msg).await;
                        if let StateRole::Leader(ref mut leader_state) = self.role {
                            if Instant::now() >= leader_state.heartbeat_due {
                                self.replicate_log().await?;
                            }
                        }
                    }
                }

                Err(_) => {
                    self.tick().await;
                }
            };
        }
    }

    async fn handle_put_msg(&mut self, msg: PutMessage) {
        if let StateRole::Leader(state) = self.role.clone() {
            let log_entry = LogEntry::new(self.term, &msg.key, &msg.value);
            self.log.append_new_entry(log_entry);
            let resp = Message::Ok(OkMessage {
                src: self.id.clone(),
                dst: msg.src.clone(),
                msg_type: "ok".into(),
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
                mid: msg.mid.clone(),
                value: Some(msg.value.clone()), //self.log.get_value(&msg.key),
            });
            // println!("Leader given PUT msg: {:?}", &msg);
            // println!("Leader returning PUT: {}", &resp);
            let _ = self.send(resp).await;
        } else {
            let msg = Message::Redirect(RedirectMessage {
                src: self.id.clone(),
                dst: msg.src.into(),
                msg_type: "redirect".into(),
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
                mid: msg.mid.into(),
            });
            let _ = self.conn.send_message(msg).await;
        }
    }

    async fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Hello(HelloMessage {
                src,
                dst,
                mid,
                leader,
            }) => {
                let leader: String = if leader == self.leader {
                    leader
                } else {
                    self.leader.clone()
                };

                let msg = Message::Ok(crate::OkMessage {
                    src: self.id.clone(),
                    dst: dst,
                    msg_type: "ok".into(),
                    leader: leader,
                    mid: mid.clone(),
                    value: None,
                });
                let _ = self.send(msg).await;
            }
            Message::Put(put) => {
                self.handle_put_msg(put).await;
            }
            Message::Get(GetMessage {
                src,
                dst,
                leader,
                mid,
                key,
                ..
            }) => {
                if let StateRole::Leader(state) = self.role.clone() {
                    let msg = Message::Ok(OkMessage {
                        src: self.id.clone(),
                        dst: src.clone(),
                        msg_type: "ok".into(),
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
                        mid: mid.clone(),
                        value: self.log.get_value(&key),
                    });
                    // println!("Leader given GET msg: {}", &key);
                    // println!("Sending back GET msg: {}", &msg);
                    let _ = self.send(msg).await;
                } else {
                    let msg = Message::Redirect(RedirectMessage {
                        src: self.id.clone(),
                        dst: src.into(),
                        msg_type: "redirect".into(),
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
                    println!("Sending redirect: {}", msg);
                    let _ = self.conn.send_message(msg).await;
                }
            }
            Message::RequestVote(msg) => {
                self.process_election_request(msg).await;
            }
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
        let cand_log_term_valid = msg.last_log_term >= self.log.get_last_term();
        let cand_log_is_longer = msg.last_log_term == self.log.get_last_term()
            && msg.last_log_idx >= self.log.get_last_idx();
        let log_is_valid = cand_log_term_valid || cand_log_is_longer;

        let up_to_date = msg.candidate_term >= self.term;
        println!("The value of cand_log_term_valid {}", &cand_log_term_valid);
        println!("The value of cand_log_is_longer {}", &cand_log_is_longer);
        println!("The value of log_is_valid {}", &log_is_valid);
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

                if state.votes_recv.len() >= self.quorum_size() {
                    let mut follower_list_as_leader = HashMap::new();
                    self.others.iter().for_each(|peer| {
                        follower_list_as_leader.insert(
                            peer.clone(),
                            VolatileState {
                                next_index: self.log.get_last_idx(),
                                match_index: 0,
                            },
                        );
                    });
                    self.promote_to_leader(follower_list_as_leader).await;
                }
            }
        }
    }

    fn reset_to_follower(&mut self, term: Term) {
        self.term = term;
        self.role = StateRole::Follower(FollowerState { leader: None });
        self.voted_for = None;
        self.ticks = rand_jitter(None);
    }

    async fn promote_to_leader(&mut self, followers: HashMap<Peer, VolatileState>) {
        self.role = StateRole::Leader(LeaderState {
            followers: followers,
            heartbeat_due: Instant::now() + Duration::from_millis(150),
        });
        self.leader = self.id.clone();
        self.ticks = rand_heartbeat_inteval();
        println!("NEW LEADER ELECTED: {}", self.id.clone());
        let _ = self.replicate_log().await;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::{ConnInfo, Connection, ConnectionGroup, HelloMessage, Message};

    use crate::Replica;
    use anyhow::Result;
    use rand::rngs::mock;
    use tokio::sync::watch::Ref;

    struct MockConnGrp {
        src: String,
        dst: String,
        leader: String,
        mid: String,
        pub times_called: Rc<RefCell<u64>>,
    }

    impl MockConnGrp {
        fn new(times_called: Rc<RefCell<u64>>) -> MockConnGrp {
            Self {
                src: "0000".into(),
                dst: "0001".into(),
                leader: "0000".into(),
                mid: "122123".into(),
                times_called,
            }
        }
    }

    impl Connection for MockConnGrp {
        async fn capture_recv_messages(&mut self) -> Result<Message> {
            *self.times_called.borrow_mut() += 1;
            return Ok(Message::Hello(crate::HelloMessage {
                src: self.src.clone(),
                dst: self.dst.clone(),
                leader: self.leader.clone(),
                mid: self.mid.clone(),
            }));
        }

        async fn send_message(&mut self, _msg: Message) -> anyhow::Result<()> {
            *self.times_called.borrow_mut() += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_new_replica_creation() {
        let conn_info = ConnInfo::new(9090).await.unwrap();
        let conn = ConnectionGroup::new(conn_info);
        let _ = Replica::new("id".into(), vec!["1209", "1231"], conn)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_run_replica() {
        let mock_counter = Rc::new(RefCell::new(0));
        let conn = MockConnGrp::new(mock_counter.clone());
        let mut rep = Replica::new("id".into(), vec!["1209", "1231"], conn)
            .await
            .unwrap();
        let _ = rep
            .send(Message::Hello(HelloMessage {
                leader: "id".into(),
                src: "1030".into(),
                dst: "1031".into(),
                mid: "12402014".into(),
            }))
            .await;
        assert_eq!(*mock_counter.borrow(), 1u64);
        let _ = rep
            .send(Message::Hello(HelloMessage {
                leader: "id".into(),
                src: "1030".into(),
                dst: "1031".into(),
                mid: "12402014".into(),
            }))
            .await;
        assert_eq!(*mock_counter.borrow(), 2u64);
    }
}
