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
    pub async fn capture_recv_messages(&self) -> Result<Message> {
        loop {
            // Wait for socket to be ready to read from
            self.connection_info.ready_to_read().await?;

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
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Connection Guard encountered error when receiving message: {e}"
                    ))
                }
            }
        }
    }
    pub async fn send_message(&self, msg: Message) -> Result<()> {
        let message = serde_json::to_string(&msg).expect("Failed to parse json value to string");
        self.connection_info.send_msg(message.as_bytes()).await?;
        Ok(())
    }
}
