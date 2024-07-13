use std::error::Error;

use clap::Parser;
use postcard::{from_bytes, to_slice};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use memryze::Message;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let listener = TcpListener::bind(args.addr).await?;

    loop {
        let (mut stream, addr) = listener.accept().await?;

        tokio::spawn(async move {
            println!("handling peer {addr}");
            let mut in_buf = vec![0; 2048];
            let n = stream.read(&mut in_buf).await.unwrap();
            if n == 0 {
                println!("client closed the connection");
                return;
            }

            let first_msg: Message = from_bytes(&in_buf[0..n]).unwrap();
            let Message::Handshake { version } = first_msg else {
                println!("First message was not handshake");
                return;
            };

            println!("client handshake version: {version}");

            let mut out_buf = vec![0; 2048];
            let used = to_slice(&first_msg, &mut out_buf).unwrap();
            println!("handshake bytes len: {}", used.len());
            stream.write_all(used).await.unwrap();
        });
    }
}
