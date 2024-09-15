// use std::fmt::{Debug, Display};
// use std::io;

pub mod db;

// pub type Result<T> = std::result::Result<T, Error>;
//
// #[derive(Debug)]
// pub enum Error {
//     Io(io::Error),
//     StreamClosed,
//     Prot(postcard::Error),
//     Other(String),
// }
//
// impl Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::Io(err) => write!(f, "I/O error: {}", err),
//             Error::StreamClosed => write!(f, "Peer closed the stream"),
//             Error::Prot(err) => write!(f, "(de)serializing error: {}", err),
//             Error::Other(str) => f.write_str(str),
//         }
//     }
// }
//
// impl std::error::Error for Error {}
//
// impl From<io::Error> for Error {
//     fn from(err: io::Error) -> Self {
//         Error::Io(err)
//     }
// }
//
// impl From<postcard::Error> for Error {
//     fn from(err: postcard::Error) -> Self {
//         Error::Prot(err)
//     }
// }
