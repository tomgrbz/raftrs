use serde::{Serialize, Serializer};

#[derive(Copy, Clone)]
pub enum MessageType {
    Get,
    Put,
    Ok,
    Fail,
    RequestVote,
    AppendEntries,
    HeartBeat,
    Hello,
    Base,
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
            MessageType::Hello => serializer.collect_str("hello"),
            MessageType::Base => serializer.collect_str("base"),
        }
    }
}
