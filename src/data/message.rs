use crate::MessageType;
use crate::Peer;
use serde::{Deserialize, Serialize, Serializer};
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RecvMessage {
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
    BaseMessage {
        src: String,
        dst: String,
        leader: String,
        mid: String,
    },
}

#[derive(Serialize)]
pub struct Message {
    mid: String,
    src: String,
    dst: String,
    leader: Peer,
    #[serde(rename = "type")]
    msg_type: MessageType,
    context: Vec<Context>,
}

pub struct Context(String, String);

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v = vec![self.0.clone(), self.1.clone()];
        serializer.collect_seq(v)
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MessageType::Get => serializer.collect_str("get"),
            MessageType::Put => serializer.collect_str("put"),
            MessageType::Ok => serializer.collect_str("ok"),
            MessageType::Fail => serializer.collect_str("fail"),
            MessageType::RequestVote => serializer.collect_str("request_vote"),
            MessageType::AppendEntries => serializer.collect_str("append_entries"),
            MessageType::HeartBeat => serializer.collect_str("heartbeat"),
        }
    }
}
impl Default for Message {
    fn default() -> Self {
        todo!()
    }
}
