use std::fmt::Debug;
use std::io::ErrorKind;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::trace;

use crate::{Error, Message, Result};

pub async fn read_msg<'a>(stream: &mut TcpStream, to_buf: &'a mut [u8]) -> Result<Message<'a>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QA;

    #[test]
    fn test_quiz() {
        let quiz = vec![
            QA {
                q: "q1".to_owned(),
                a: "a1".to_owned(),
            },
            QA {
                q: "q2".to_owned(),
                a: "a2".to_owned(),
            },
            QA {
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
