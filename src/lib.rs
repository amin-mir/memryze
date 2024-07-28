use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub mod db;
pub mod protocol;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message<'a> {
    Handshake { version: u8 },

    AddQA { q: &'a str, a: &'a str },
    AddQAResp,

    GetQuiz,
    Quiz { count: u16, qas_bytes: &'a [u8] },

    InternalError,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct QA {
    pub q: String,
    pub a: String,
}

fn hex(data: &[u8]) -> String {
    let mut enc = String::new();
    for byte in data {
        enc.push_str(&format!("0x{:02X} ", byte));
    }
    // Remove the space at the end.
    if enc.len() > 0 {
        enc.pop();
    }
    enc
}
