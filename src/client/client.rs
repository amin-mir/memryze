use std::error::Error;
use std::process;

use postcard::{from_bytes, to_slice};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use memryze::Message;

const ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(ADDR).await?;

    let handshake = Message::Handshake { version: 1 };

    let mut in_buf = vec![0; 2048];
    let mut out_buf = vec![0; 2048];

    let used = to_slice(&handshake, &mut out_buf).unwrap();
    stream.write_all(used).await?;

    let n = stream.read(&mut in_buf).await?;
    if n == 0 {
        println!("Server closed the connection");
    }

    println!("Handshake response");
    print_hex(&in_buf[0..n]);

    let handshake_reply = from_bytes(&in_buf[0..n]).unwrap();
    let Message::Handshake { version } = handshake_reply else {
        println!("Handshake reply has the wrong type: {:?}", handshake_reply);
        process::exit(1);
    };

    println!("Server handshake version: {version}");

    let msg = Message::AddQA {
        q: &"I'm looking for a man in finance",
        a: &"Etsin miestä rahoitusalalta",
    };
    let used = to_slice(&msg, &mut out_buf).unwrap();
    stream.write_all(used).await?;

    let n = stream.read(&mut in_buf).await?;
    if n == 0 {
        println!("Server closed the connection");
    }

    println!("AddQA response");
    print_hex(&in_buf[0..n]);

    Ok(())
}

fn print_hex(data: &[u8]) {
    for byte in data {
        print!("0x{:02X} ", byte);
    }
    println!();
}
