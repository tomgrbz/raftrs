use clap::Parser;
use raft_rs::RecvMessage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::time::{Duration, Instant};
use std::{
    net::{SocketAddr, UdpSocket},
    thread,
};

const BROADCAST: &str = "FFFF";

fn main() {
    let cli = Cli::parse();
    let rep = Replica::new(cli.port, cli.id, cli.others);
    rep.run();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    port: u16,
    id: String,
    others: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum StateRole {
    /// The node is a follower of the leader.
    #[default]
    Follower,
    /// The node could become a leader.
    Candidate,
    /// The node is a leader.
    Leader,
    /// The node could become a candidate, if `prevote` is enabled.
    PreCandidate,
}

struct Replica {
    port: u16,
    id: String,
    role: StateRole,
    others: Vec<String>,
    leader_id: String,
    socket: UdpSocket,
    addr: SocketAddr,
    heartbeat_elapsed: u8,
    election_elapsed: u8,
}

struct Peer {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HelloMessage {
    src: String,
    dst: String,
    leader: String,
    #[serde(rename = "type")]
    msg_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetMessage {
    src: String,
    dst: String,
    leader: String,
    #[serde(rename = "type")]
    msg_type: String,
    mid: String,
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PutMessage {
    src: String,
    dst: String,
    leader: String,
    #[serde(rename = "type")]
    msg_type: String,
    mid: String,
    key: String,
    value: String,
}

impl Replica {
    fn new(port: u16, id: String, others: Vec<String>) -> Self {
        let socket =
            UdpSocket::bind(("localhost", port)).expect("Unable to create udp socket on localhost");
        let addr = socket.local_addr().expect("Failed to fetch local address");
        println!("Successfully created Replica");
        //let others: Vec<String> = others.into_iter().map(|element| element.into()).collect();
        let role = {
            if id == "0000" {
                StateRole::Leader
            } else {
                StateRole::default()
            }
        };
        let rep = Replica {
            port,
            id,
            role,
            leader_id: "0000".into(),
            others,
            socket,
            addr,
            election_elapsed: 0,
            heartbeat_elapsed: 0,
        };
        println!("Created Replica object");
        let msg = json!({
            "src": rep.id,
            "dst": BROADCAST,
            "leader": BROADCAST,
            "type":"hello"
        });
        rep.send(msg);
        rep
    }

    fn send(&self, message: Value) -> Result<usize, String> {
        let message =
            serde_json::to_string(&message).expect("Failed to parse json value to string");
        match self.socket.send_to(message.as_bytes(), self.addr) {
            Ok(payload_size) => Ok(payload_size),
            Err(_) => {
                println!("Could not send message on socket.");
                Err("Could not send message.".into())
            }
        }
    }
    fn tick(&mut self) {
        if self.role == StateRole::Leader {
            self.tick_heartbeat();
        } else {
            self.tick_election();
        }
    }
    fn tick_heartbeat(&mut self) {
        self.heartbeat_elapsed += 1;
        self.election_elapsed += 1;

        self.handle_message(RecvMessage::AppendEntriesMessage)
    }

    fn tick_election(&mut self) {
        self.election_elapsed += 1;
    }

    fn run(mut self) {
        let mut packet_as_bytes = [0; 1024];
        let _t = Instant::now();
        loop {
            let t = Instant::now();
            thread::sleep(Duration::from_millis(10));
            match self.socket.recv_from(&mut packet_as_bytes) {
                Ok((recv_size, peer_addr)) => {
                    let filled_buffer = &mut packet_as_bytes[..recv_size];
                    //let msg = from_utf8(filled_buffer).unwrap();//serde_json::from_slice(filled_buffer).unwrap();
                    let msg: RecvMessage = serde_json::from_slice(filled_buffer).unwrap();
                    //let msg_str = from_utf8(msg).unwrap();
                    let peer_port = peer_addr.port();

                    println!("Received msg: {:?} bytes from peer: {peer_port}", msg);
                    self.handle_message(msg);

                    if t.elapsed() >= Duration::from_millis(100) {
                        self.tick();
                    }
                }
                Err(e) => {
                    println!("Encountered error: {e}")
                }
            }
        }
    }

    fn handle_message(&self, msg: RecvMessage) {
        match msg {
            RecvMessage::HelloMessage { src, dst: _, leader: _ } => {
                let msg = json!({
                    "src": self.id,
                    "dst": src,
                    "leader": BROADCAST,
                    "type":"ok"
                });
                self.send(msg).expect("Failed to send back to put");
            }
            RecvMessage::GetMessage {
                src,
                dst: _,
                leader: _,
                mid,
                key: _,
            } => {
                let msg = json!({
                    "src": self.id,
                    "dst": src,
                    "leader": BROADCAST,
                        "MID": mid,
                    "type":"fail"
                });
                self.send(msg).expect("Failed to send back to put");
            }
            RecvMessage::PutMessage {
                src,
                dst: _,
                leader: _,
                mid,
                key: _,
                value: _,
            } => {
                let msg = json!({
                    "src": self.id,
                    "dst": src,
                    "leader": "FFFF",
                            "MID": mid,
                    "type":"fail"
                });
                self.send(msg).expect("Failed to send back to put");
            }
            RecvMessage::AppendEntriesMessage => {
                if self.role == StateRole::Leader {
                    for rep in self.others.iter() {
                        let msg = json!({
                            "src": self.id,
                            "dst": rep,
                            "leader": self.id,
                            "type":"append_entries"
                        });
                        println!("Sending append heartbeat to children: {rep}");
                        self.send(msg).expect("Failed to send back to heartbeat");
                    }
                } else {
                    let msg = json!({
                        "src": self.id,
                        "dst": "0000",
                        "leader": "0000",
                        "type":"append_entries"
                    });
                    self.send(msg).expect("Failed to send back to heartbeat");
                }
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Replica;

    #[test]
    fn test_new_replica() {
        Replica::new(9090, "id".into(), vec!["1209".into(), "1231".into()]);
    }
}
