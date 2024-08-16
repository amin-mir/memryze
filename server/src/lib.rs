use std::fmt::{Debug, Display};
use std::io;

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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    StreamClosed,
    Prot(postcard::Error),
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::StreamClosed => write!(f, "Peer closed the stream"),
            Error::Prot(err) => write!(f, "(de)serializing error: {}", err),
            Error::Other(str) => f.write_str(str),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Error::Prot(err)
    }
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
