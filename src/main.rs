use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::from_utf8;
use std::{
    io::{Read, Write},
    net::{SocketAddr, UdpSocket},
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
    port: u32,
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
    port: u32,
    id: String,
    role: StateRole,
    others: Vec<String>,
    leader_id: String,
    socket: UdpSocket,
    addr: SocketAddr,
    heartbeat_tick: u8,
    election_tick: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Message {
    #[serde(rename = "hello")]
    HelloMessage {
        src: String,
        dst: String,
        leader: String,
    },
    #[serde(rename = "get")]
    GetMessage {
        src: String,
        dst: String,
        leader: String,
        mid: String,
        key: String,
    },
    #[serde(rename = "put")]
    PutMessage {
        src: String,
        dst: String,
        leader: String,
        mid: String,
        key: String,
        value: String,
    },
    #[serde(rename = "ok")]
    OkMessage,
    #[serde(rename = "fail")]
    FailMessage,
    #[serde(rename = "append_entries")]
    AppendEntriesMessage,
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
    fn new(port: u32, id: String, others: Vec<String>) -> Self {
        let socket =
            UdpSocket::bind("127.0.0.1:0").expect("Unable to create udp socket on localhost");
        let addr = socket.local_addr().expect("Failed to fetch local address");
        println!("Succesfully created Replica");
        //let others: Vec<String> = others.into_iter().map(|element| element.into()).collect();
        let role = {if id == "0000" {
            StateRole::Leader
        } else {
            StateRole::default()
        }};
        let mut rep = Replica {
            port,
            id: id.into(),
            role,
            leader_id: "0000".into(),
            others,
            socket,
            addr,
        };
        println!("Created Replica object");
        let msg = json!({
            "src": rep.id,
            "dst": "FFFF",
            "leader": "FFFF",
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
    fn run(self) {
        let mut packet_as_bytes = [0; 1024];
        loop {
            match self.socket.recv_from(&mut packet_as_bytes) {
                Ok((recv_size, peer_addr)) => {
                    let filled_buffer = &mut packet_as_bytes[..recv_size];
                    //let msg = from_utf8(filled_buffer).unwrap();//serde_json::from_slice(filled_buffer).unwrap();
                    let msg: Message = serde_json::from_slice(filled_buffer).unwrap();
                    //let msg_str = from_utf8(msg).unwrap();
                    let peer_port = peer_addr.port();

                    // match msg.msg_type {
                    //     MessageType::HelloMessage(msg_type) => todo!(),
                    //     MessageType::GetMessage(msg_type) => todo!(),
                    //     MessageType::PutMessage(msg_type) => todo!(),
                    //
                    // }
                    println!("Received msg: {:?} bytes from peer: {peer_port}", msg);
                    let _ = self.handle_message(msg);
                }
                Err(e) => {
                    println!("Encountered error: {e}")
                }
            }
        }
    }

    fn handle_message(&self, msg: Message) {
        match msg {
            Message::HelloMessage { src, dst, leader } => {
                let msg = json!({
                    "src": self.id,
                    "dst": src,
                    "leader": "FFFF",
                    "type":"ok"
                });
                self.send(msg).expect("Failed to send back to put");
            }
            Message::GetMessage {
                src,
                dst,
                leader,
                mid,
                key,
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
            Message::PutMessage {
                src,
                dst,
                leader,
                mid,
                key,
                value,
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
            Message::AppendEntriesMessage => {
                if self.role == StateRole::Leader {
                    for rep in self.others.iter() {
                        let msg = json!({
                    "src": self.id,
                    "dst": rep,
                    "leader": self.id,
                    "type":"fail"
                });
                        self.send(msg).expect("Failed to send back to heartbeat");
                    }
                } else {
                    let msg = json!({
                    "src": self.id,
                    "dst": "0000",
                    "leader": self.id,
                    "type":"fail"
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
