use serde::{Deserialize, Serialize};
use std::fmt::Display;

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HelloMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PutMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
    pub key: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OkMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FailMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RedirectMessage {
    pub src: String,
    pub dst: String,
    pub leader: String,
    #[serde(rename = "MID")]
    pub mid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "hello")]
    Hello(HelloMessage),
    #[serde(rename = "get")]
    Get(GetMessage),
    #[serde(rename = "put")]
    Put(PutMessage),
    #[serde(rename = "ok")]
    Ok(OkMessage),
    #[serde(rename = "fail")]
    Fail(FailMessage),
    #[serde(rename = "append_entries")]
    AppendEntriesMessage,
    #[serde(rename = "redirect")]
    Redirect(RedirectMessage),
}

// impl Message {
//     pub fn new(
//         mid: Option<String>,
//         src: String,
//         dst: String,
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
