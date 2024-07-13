use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<'a> {
    Handshake { version: u8 },
    AddQA { q: &'a [u8], a: &'a [u8] },
}
