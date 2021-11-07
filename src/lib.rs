//! Asynchronous [asar] parser using [tokio].
//! 
//! Currently unpacked files and directories are not supported. Things other than
//! this should just work fine.
//! 
//! [asar]: https://github.com/electron/asar
//! [tokio]: https://tokio.rs

mod archive;
mod header;

pub use archive::{Archive, File};
pub use header::{Integrity, Algorithm};
