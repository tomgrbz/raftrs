use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{Log, LogEntry, Term};

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HelloMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub key: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PutMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub key: &'a str,
    pub value: &'a str,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OkMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FailMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RedirectMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AppendEntriesMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub term: Term,
    pub prev_log_term: Term,
    pub prev_log_index: usize,
    pub entries: Vec<LogEntry>,
    pub leader_commit_idx: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AppendEntriesMessageResponse<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub term: Term,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequestVoteMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub candidate_term: Term,
    pub last_log_idx: usize,
    pub last_log_term: Term,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequestVoteResponseMessage<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub leader: &'a str,
    #[serde(rename = "MID")]
    pub mid: &'a str,
    pub term: Term,
    pub voted_granted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Message<'a> {
    #[serde(rename = "hello")]
    Hello(HelloMessage<'a>),
    #[serde(rename = "get")]
    Get(GetMessage<'a>),
    #[serde(rename = "put")]
    Put(PutMessage<'a>),
    #[serde(rename = "ok")]
    Ok(OkMessage<'a>),
    #[serde(rename = "fail")]
    Fail(FailMessage<'a>),
    #[serde(rename = "append_entries")]
    AppendEntries(AppendEntriesMessage<'a>),
    #[serde(rename = "append_entries_response")]
    AppendEntriesResponse(AppendEntriesMessageResponse<'a>),
    #[serde(rename = "redirect")]
    Redirect(RedirectMessage<'a>),
    #[serde(rename = "request_vote")]
    RequestVote(RequestVoteMessage<'a>),
    #[serde(rename = "request_vote_response")]
    RequestVoteResponse(RequestVoteResponseMessage<'a>),
}

impl<'a> Message<'a> {
    pub fn get_src(&'a self) -> &'a str {
        match self {
            Message::Hello(msg) => &msg.src,
            Message::Get(msg) => &msg.src,
            Message::Put(msg) => &msg.src,
            Message::Ok(msg) => &msg.src,
            Message::Fail(msg) => &msg.src,
            Message::AppendEntries(msg) => &msg.src,
            Message::AppendEntriesResponse(msg) => &msg.src,
            Message::Redirect(msg) => &msg.src,
            Message::RequestVote(msg) => &msg.src,
            Message::RequestVoteResponse(msg) => &msg.src,
        }
    }
}

// impl Message {
//     pub fn new(
//         mid: Option<String>,
//         src: &'a str,
//         dst: &'a str,
//         leader: Peer,
//         msg_type: MessageType,
//         context: Contexts,
//     ) -> Self {

//             mid,
//             src,
//             dst,
//             leader,
//             msg_type,
//             context,
//         }
//     }

//     pub fn build_from_recv(received_msg: RecvMessage) -> Self {
//         let mut final_msg = Message::default();
//         match received_msg {
//             RecvMessage::HelloMessage { src, dst, leader, mid } => {
//                 final_msg.src = dst;
//                 final_msg.dst = src;
//                 final_msg.leader = Peer::new(leader);
//                 final_msg.mid = Some(mid);
//                 final_msg
//             }
//             RecvMessage::GetMessage {
//                 src,
//                 dst,
//                 mid,
//                 leader,
//                 ..
//             } => {
//                 final_msg.src = dst;
//                 final_msg.dst = src;
//                 final_msg.mid = Some(mid);
//                 final_msg.leader = Peer::new(leader);
//                 final_msg
//             }
//             RecvMessage::PutMessage {
//                 src,
//                 dst,
//                 leader,
//                 mid,
//                 ..
//             } => {
//                 final_msg.src = dst;
//                 final_msg.dst = src;
//                 final_msg.mid = Some(mid);
//                 final_msg.leader = Peer::new(leader);
//                 final_msg
//             }
//             RecvMessage::BaseMessage { src, dst, leader } => {
//                 final_msg.src = dst;
//                 final_msg.dst = src;
//                 final_msg.leader = Peer::new(leader);

//                 final_msg
//             }
//             _ => final_msg,
//         }
//     }

//     pub fn set_msg_type(&mut self, msg_type: MessageType) {
//         self.msg_type = msg_type
//     }

//     pub fn get_msg_type(&self) -> MessageType {
//         self.msg_type
//     }

//     pub fn set_leader(&mut self, leader: &Peer) {
//         self.leader = leader.clone();
//     }
//     pub fn get_leader(&self) -> &Peer {
//         &self.leader
//     }
// }
mod tests {

    use crate::data::message::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_hello_deserialize() {
        let hello_struct = Message::Hello(HelloMessage {
            src: "0000".into(),
            dst: "ffff".into(),
            leader: "0000".into(),
            mid: "Random".into(),
        });
        assert_tokens(
            &hello_struct,
            &[
                Token::Struct {
                    name: "HelloMessage",
                    len: 5,
                },
                Token::String("type"),
                Token::String("hello"),
                Token::String("src"),
                Token::String("0000"),
                Token::String("dst"),
                Token::String("ffff"),
                Token::String("leader"),
                Token::String("0000"),
                Token::String("MID"),
                Token::String("Random"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_ok_serialize_with_no_value() {
        let hello_struct = Message::Ok(OkMessage {
            src: "0000".into(),
            dst: "ffff".into(),
            leader: "0000".into(),
            mid: "Random".into(),
            value: None,
        });
        assert_tokens(
            &hello_struct,
            &[
                Token::Struct {
                    name: "OkMessage",
                    len: 5,
                },
                Token::String("type"),
                Token::String("ok"),
                Token::String("src"),
                Token::String("0000"),
                Token::String("dst"),
                Token::String("ffff"),
                Token::String("leader"),
                Token::String("0000"),
                Token::String("MID"),
                Token::String("Random"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_ok_serialize_with_given_value() {
        let hello_struct = Message::Ok(OkMessage {
            src: "0000".into(),
            dst: "ffff".into(),
            leader: "0000".into(),
            mid: "Random".into(),
            value: Some("89".into()),
        });
        assert_eq!(
            serde_json::to_string(&hello_struct).unwrap(),
            r#"{"type":"ok","src":"0000","dst":"ffff","leader":"0000","MID":"Random","value":"89"}"#
        );
        assert_tokens(
            &hello_struct,
            &[
                Token::Struct {
                    name: "OkMessage",
                    len: 6,
                },
                Token::String("type"),
                Token::String("ok"),
                Token::String("src"),
                Token::String("0000"),
                Token::String("dst"),
                Token::String("ffff"),
                Token::String("leader"),
                Token::String("0000"),
                Token::String("MID"),
                Token::String("Random"),
                Token::String("value"),
                Token::Some,
                Token::String("89"),
                Token::StructEnd,
            ],
        );
    }
}
