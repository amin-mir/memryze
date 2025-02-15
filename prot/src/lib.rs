use std::fmt::{Debug, Display};
use std::io::{self, ErrorKind};

use message::Message;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::trace;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    StreamClosed,
    Prot(postcard::Error),
    Other(anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::StreamClosed => write!(f, "Peer closed the stream"),
            Error::Prot(err) => write!(f, "(de)serializing error: {}", err),
            Error::Other(err) => write!(f, "{}", err),
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

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err)
    }
}

pub async fn read_msg<'a>(stream: &mut TcpStream, to_buf: &'a mut [u8]) -> Result<Message<'a>> {
    // TODO: return error if to_buf is not large enough to read the message.
    let n = stream.read(to_buf).await?;
    if n == 0 {
        return Err(Error::StreamClosed);
    }

    trace!(hex = crate::hex(&to_buf[0..n]), "Received message");

    postcard::from_bytes(&to_buf[0..n]).map_err(Into::into)
}

pub async fn write_msg(
    stream: &mut TcpStream,
    from_buf: &mut [u8],
    msg: &Message<'_>,
) -> Result<()> {
    let used = postcard::to_slice(msg, from_buf)?;

    trace!(hex = crate::hex(used), "Sending message");

    match stream.write_all(used).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::WriteZero => Err(Error::StreamClosed),
        Err(err) => Err(err.into()),
    }
}

pub fn ser_slice<'a, T>(data: &'a [T], dest: &'a mut [u8]) -> Result<&'a [u8]>
where
    T: Serialize,
{
    let mut buf_used = 0;
    for i in 0..data.len() {
        let used = postcard::to_slice(&data[i], &mut dest[buf_used..])?;
        buf_used += used.len();
    }

    Ok(&dest[0..buf_used])
}

pub fn deser_from_bytes<'a, T>(mut src: &'a [u8], count: u16, dest: &mut Vec<T>) -> Result<()>
where
    T: Deserialize<'a> + Debug,
{
    for _ in 0..count {
        let (data, unused) = postcard::take_from_bytes::<T>(src)?;
        dest.push(data);
        src = unused;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use message::QA;

    #[test]
    fn test_quiz() {
        let quiz = vec![
            QA {
                id: 1,
                q: "q1".to_owned(),
                a: "a1".to_owned(),
            },
            QA {
                id: 2,
                q: "q2".to_owned(),
                a: "a2".to_owned(),
            },
            QA {
                id: 3,
                q: "q3".to_owned(),
                a: "a3".to_owned(),
            },
        ];

        let mut ser_buf = vec![0u8; 512];
        let qas_bytes = ser_slice(&quiz, &mut ser_buf[..]).unwrap();

        let mut msg_buf = vec![0u8; 512];
        let msg = Message::Quiz {
            count: 3,
            qas_bytes,
        };
        let used = postcard::to_slice(&msg, &mut msg_buf).unwrap();

        let msg = postcard::from_bytes(used).unwrap();
        let Message::Quiz { count, qas_bytes } = msg else {
            panic!("deserialized wrong type of Message: {:?}", msg);
        };

        assert_eq!(count, 3);
        let mut qas: Vec<QA> = Vec::with_capacity(count as usize);
        deser_from_bytes(qas_bytes, count, &mut qas).unwrap();

        assert_eq!(qas, quiz);
    }
}
