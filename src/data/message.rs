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
impl Message {
    pub fn new(mid: String, src: String, dst: String, leader: Peer, msg_type: MessageType, context: Vec<Context>) -> Message {
        Message {
            mid,
            src,
            dst,
            leader,
            msg_type,
            context
        }
    }
}
impl Default for Message {
    fn default() -> Self {
        todo!()
    }
}


#[derive(Clone)]
pub struct Context(String, String);

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(self.clone().into_iter())
    }
}

impl IntoIterator for Context {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        vec![(self.0.clone(), self.1.clone())].into_iter()
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
