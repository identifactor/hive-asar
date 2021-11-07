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
  pub(crate) fn list(&mut self) -> impl Iterator<Item = String> + '_ {
    self
      .files
      .iter_mut()
      .map(|(name, entry)| -> Box<dyn Iterator<Item = _>> {
        match entry {
          HeaderEntry::File(_metadata) => Box::from(std::iter::once(name.clone())),
          HeaderEntry::Dir(dir) => Box::from(dir.list().map(|y| name.to_string() + "/" + &y))
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Integrity {
  pub algorithm: Algorithm,
  pub hash: String,
  #[serde(rename = "blockSize")]
  pub block_size: u32,
  pub blocks: Vec<String>,
  #[serde(skip)]
  _priv: ()
}

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
