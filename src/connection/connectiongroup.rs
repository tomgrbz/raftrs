use std::io;

use crate::connection::ConnInfo;
use crate::Message;
use anyhow::{anyhow, Result};

/// The boundary checker holding connection info and the socket for a Raft Replica.
/// Responsible for receiving packets on a UDP connection and then sending those as deserialized
/// `Message` types to the main Replica
#[derive(Debug)]
pub struct ConnectionGroup {
    connection_info: ConnInfo,
}

pub trait Connection {
    async fn capture_recv_messages(&mut self) -> Result<Message>;

    async fn send_message(&mut self, msg: Message) -> Result<()>;
}

#[derive(thiserror::Error, Debug)]
pub enum ConnError {
    #[error("Failed to deserialize message")]
    DeErr,
    #[error("Failed to serialize message")]
    SeErr,
}

impl ConnectionGroup {
    pub fn new(conn_info: ConnInfo) -> Self {
        ConnectionGroup {
            connection_info: conn_info,
        }
    }
}
impl Connection for ConnectionGroup {
    async fn capture_recv_messages(&mut self) -> Result<Message> {
        // Wait for socket to be ready to read from
        
        self.connection_info.ready_to_read().await?;
        
        dbg!("made it past ready_to_read");

        let mut packet_as_bytes = vec![0; 1024];

        match self.connection_info.recv_msg(&mut packet_as_bytes) {
            Ok(recv_size) => {
                let filled_buffer = &mut packet_as_bytes[..recv_size];
                let msg = serde_json::from_slice::<Message>(filled_buffer);

                match msg {
                    Ok(message) => {
                        return Ok(message);
                    }
                    Err(e) => return Err(ConnError::DeErr.into()),
                };
            }
            Err(e) => {
                return Err(anyhow!(
                    "Connection Guard encountered error when receiving message: {e}"
                ))
            }
        }
    }
    async fn send_message(&mut self, msg: Message) -> Result<()> {
        let message = serde_json::to_string(&msg).expect("Failed to parse json value to string");
        println!("Sent msg from {}", msg.get_src());
        self.connection_info.send_msg(message.as_bytes()).await?;
        Ok(())
    }
}
