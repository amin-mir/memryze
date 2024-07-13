use std::error::Error;
use std::process;

use postcard::{from_bytes, to_slice};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use memryze::Message;

const ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Client running");
    let mut stream = TcpStream::connect(ADDR).await?;

    let handshake = Message::Handshake { version: 1 };

    let mut in_buf = vec![0; 2048];
    let mut out_buf = vec![0; 2048];

    let used = to_slice(&handshake, &mut out_buf).unwrap();
    stream.write_all(used).await?;

    let n = stream.read(&mut in_buf).await?;
    if n == 0 {
        println!("server closed the connection");
    }

    println!("handshake reply");
    print_hex(&in_buf[0..n]);

    let handshake_reply = from_bytes(&in_buf[0..n]).unwrap();
    let Message::Handshake { version } = handshake_reply else {
        println!("handshake reply has the wrong type: {:?}", handshake_reply);
        process::exit(1);
    };

    println!("server handshake version: {version}");

    Ok(())
}

fn print_hex(data: &[u8]) {
    for byte in data {
        print!("0x{:02X} ", byte);
    }
    println!();
}
