use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::trace;

pub mod db;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message<'a> {
    Handshake { version: u8 },

    AddQA { q: &'a str, a: &'a str },
    AddQAResp,

    GetQuiz,
    Quiz { count: usize, qa_bytes: &'a [u8] },

    InternalError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct QA<'a> {
    q: &'a str,
    a: &'a str,
}

// The reason I've created a struct to contain the input and output buffer
// is that it helps with choosing the right buffer when reading/writing.
pub struct Protocol {
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
}

impl Protocol {
    pub fn new(cap: usize) -> Self {
        Self {
            in_buf: vec![0; cap],
            out_buf: vec![0; cap],
        }
    }
    pub async fn read_msg(&mut self, stream: &mut TcpStream) -> anyhow::Result<Message> {
        let n = stream.read(&mut self.in_buf).await?;
        if n == 0 {
            anyhow::bail!("Peer closed the connection");
        }

        trace!(hex = hex(&self.in_buf[0..n]), "Received message");

        postcard::from_bytes(&self.in_buf[0..n]).map_err(Into::into)
    }

    pub async fn write_msg(
        &mut self,
        stream: &mut TcpStream,
        msg: &Message<'_>,
    ) -> anyhow::Result<()> {
        let used = postcard::to_slice(msg, &mut self.out_buf)?;
        stream.write_all(used).await.map_err(Into::into)
    }
}

fn hex(data: &[u8]) -> String {
    let mut hex = String::new();
    for byte in data {
        hex.push_str(&format!("0x{:02X} ", byte));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiz() {
        let quiz = vec![
            QA { q: "q1", a: "a1" },
            QA { q: "q2", a: "a2" },
            QA { q: "q3", a: "a3" },
        ];

        let mut msg_buf = vec![0u8; 512];
        let mut qa_bytes = vec![0u8; 512];

        // TODO: extract SERIALIZE to a function.
        let mut buf_used = 0;
        for i in 0..3 {
            let used = postcard::to_slice(&quiz[i], &mut qa_bytes[buf_used..]).unwrap();
            buf_used += used.len();
        }

        let msg = Message::Quiz {
            count: 3,
            qa_bytes: &qa_bytes[0..buf_used],
        };

        let used = postcard::to_slice(&msg, &mut msg_buf).unwrap();

        // TODO: extract DESERIALIZE to a function.
        let msg = postcard::from_bytes(used).unwrap();
        let Message::Quiz {
            count,
            mut qa_bytes,
        } = msg
        else {
            panic!("deserialized wrong type of Message: {:?}", msg);
        };

        assert_eq!(count, 3);
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            let (qa, unused) = postcard::take_from_bytes::<QA>(qa_bytes).unwrap();
            res.push(qa);
            qa_bytes = unused;
        }

        assert_eq!(res, quiz);
        println!("{:?}", res);
    }
}
