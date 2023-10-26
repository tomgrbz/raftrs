use clap::Parser;
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{SocketAddr, UdpSocket},
};
use serde::{Deserialize, Serialize};

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

struct Replica {
    port: u32,
    id: String,
    others: Vec<String>,
    socket: UdpSocket,
    addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
struct HelloMessage {
    src: String,
    dst: String,
    leader: String,
    r#type: String,
}

impl Replica {
    fn new(port: u32, id: String, others: Vec<String>) -> Self {
        let socket =
            UdpSocket::bind("127.0.0.1:0").expect("Unable to create udp socket on localhost");
        let addr = socket.local_addr().expect("Failed to fetch local address");
        println!("Succesfully created Replica");
        //let others: Vec<String> = others.into_iter().map(|element| element.into()).collect();
        let mut rep = Replica {
            port,
            id: id.into(),
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

    fn send(&mut self, message: Value) -> Result<usize, String> {
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
                    let msg: HelloMessage = serde_json::from_slice(filled_buffer).unwrap();
                    let peer_port = peer_addr.port();

                    println!("Received msg: {:?} bytes from peer: {peer_port}", msg);
                }
                Err(e) => {
                    println!("Encountered error: {e}")
                }
            }
        }
    }

    fn handle_message(self, datagram: &mut [u8; 1024]) {
        todo!()

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
