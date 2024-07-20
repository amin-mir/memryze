use std::error::Error;
use std::process;

use tokio::net::TcpStream;
// use tokio::time::{self, Duration};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use memryze::{Message, Protocol};

const ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut stream = TcpStream::connect(ADDR).await?;

    let mut prot = Protocol::new(2048);

    let handshake = Message::Handshake { version: 1 };

    prot.write_msg(&mut stream, &handshake).await?;

    let handshake_reply = prot.read_msg(&mut stream).await?;

    let Message::Handshake { version } = handshake_reply else {
        error!(?handshake_reply, "Handshake reply has the wrong type");
        process::exit(1);
    };

    info!(version, "Received handshake from server");

    let msg = Message::AddQA {
        q: &"I'm looking for a man in finance",
        a: &"Etsin miestä rahoitusalalta",
    };
    prot.write_msg(&mut stream, &msg).await?;

    let add_qa_reply = prot.read_msg(&mut stream).await?;

    info!(reply = ?add_qa_reply, "AddQA reply");

    Ok(())
}
