use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) type Header = HeaderEntry;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum HeaderEntry {
  Dir(Directory),
  File(FileMetadata)
}

impl HeaderEntry {
  pub(crate) fn search_segments(&self, segments: &[&str]) -> Option<&HeaderEntry> {
    match self {
      _ if segments.is_empty() => Some(self),
      Self::File(_) => None,
      Self::Dir(dir) => dir
        .files
        .get(segments[0])
        .and_then(|x| x.search_segments(&segments[1..]))
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Directory {
  pub(crate) files: HashMap<String, HeaderEntry>,
  #[serde(skip)]
  _priv: ()
}

impl Directory {
  pub(crate) fn file_paths(&self) -> impl Iterator<Item = String> + '_ {
    self
      .files
      .iter()
      .map(|(name, entry)| -> Box<dyn Iterator<Item = _>> {
        match entry {
          HeaderEntry::File(_) => Box::from(std::iter::once(name.clone())),
          HeaderEntry::Dir(dir) => Box::from(dir.file_paths().map(|y| name.to_string() + "/" + &y))
        }
      })
      .flatten()
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FileMetadata {
  #[serde(with = "serde_offset")]
  pub offset: u64,
  // no larger than 9007199254740991
  pub size: u64,
  #[serde(default)]
  pub executable: bool,
  pub integrity: Option<Integrity>
}

/// Checksums of a file, containing the hash for the whole file as well as hashes
/// for each block of data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Integrity {
  /// Hashing algorithm used in the file.
  /// 
  /// Currently only SHA256 is used in the file format.
  pub algorithm: Algorithm,

  /// The hash for the whole file.
  pub hash: String,

  /// Size of a block.
  #[serde(rename = "blockSize")]
  pub block_size: u32,

  /// Hashes for each block of data containing `block_size` bytes.
  pub blocks: Vec<String>,

  #[serde(skip)]
  _priv: ()
}

/// Hashing algorithm used in asar archives.
/// 
/// Currently only SHA256 is used.
#[derive(Debug, Serialize, Deserialize)]
pub enum Algorithm {
  SHA256
}

mod serde_offset {
  use serde::de::Error;
  use serde::{Deserialize, Deserializer, Serializer};

  pub fn serialize<S: Serializer>(offset: &u64, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&offset.to_string())
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<u64, D::Error> {
    u64::from_str_radix(&String::deserialize(de)?, 10).map_err(D::Error::custom)
  }
}
