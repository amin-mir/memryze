use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<'a> {
    Handshake { version: u8 },
    AddQA { q: &'a str, a: &'a str },
    AddQAResp,
    InternalError,
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
            in_buf: Vec::with_capacity(cap),
            out_buf: Vec::with_capacity(cap),
        }
    }
    pub async fn read_msg(&mut self, stream: &mut TcpStream) -> anyhow::Result<Message> {
        let n = stream.read(&mut self.in_buf).await?;
        if n == 0 {
            anyhow::bail!("client closed the connection");
        }

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
