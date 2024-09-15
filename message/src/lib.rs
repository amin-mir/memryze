use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message<'a> {
    Handshake { version: u8, token: &'a str },
    HandshakeResp,

    AddQA { q: &'a str, a: &'a str },
    AddQAResp,

    GetQuiz,
    Quiz { count: u16, qas_bytes: &'a [u8] },

    ReviewQA { id: i64, correct: bool },
    ReviewQAResp,

    InternalError,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QA {
    pub id: i64,
    pub q: String,
    pub a: String,
}
