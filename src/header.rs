use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type Header = Entry;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
  Dir(Directory),
  File(FileMetadata)
}

impl Entry {
  pub(crate) fn search_segments(&self, segments: &[&str]) -> Option<&Entry> {
    match self {
      _ if segments.is_empty() => Some(self),
      Self::File(_) => None,
      Self::Dir(dir) => dir.files.get(segments[0]).and_then(|x| x.search_segments(&segments[1..]))
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directory {
  pub files: HashMap<String, Entry>,
  #[serde(skip)]
  _priv: ()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
  #[serde(with = "serde_offset")]
  pub offset: u64,
  // no larger than 9007199254740991
  pub size: u64,
  #[serde(default)]
  pub executable: bool,
  pub integrity: Integrity,
  #[serde(skip)]
  _priv: ()
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
  use serde::{Deserializer, Deserialize, Serializer};
  use serde::de::Error;

  pub fn serialize<S: Serializer>(offset: &u64, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&offset.to_string())
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<u64, D::Error> {
    u64::from_str_radix(&String::deserialize(de)?, 10)
      .map_err(D::Error::custom)
  }
}
