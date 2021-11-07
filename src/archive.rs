use crate::header::{FileMetadata, Header, HeaderEntry, Integrity};
use std::borrow::BorrowMut;
use std::io::SeekFrom;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, ReadBuf, Take};

/// Read-only asar archive.
pub struct Archive<R: AsyncRead + AsyncSeek + Unpin> {
  offset: u64,
  header: Header,
  reader: R
}

impl<R: AsyncRead + AsyncSeek + Unpin> Archive<R> {
  /// Opens an asar archive and parse its header.
  pub async fn new(mut reader: R) -> io::Result<Self> {
    reader.seek(SeekFrom::Start(12)).await?;
    let header_size = u32::from_be(reader.read_u32().await?);
    let mut header_bytes = vec![0; header_size as _];
    reader.read_exact(&mut header_bytes).await?;
    let header = serde_json::from_slice(&header_bytes)?;
    Ok(Self {
      offset: header_size as u64 + 16,
      header,
      reader
    })
  }

  /// Returns an *synchronous* iterator over all files' paths.
  /// 
  /// Does not include directories.
  pub fn file_paths(&self) -> impl Iterator<Item = String> + '_ {
    if let HeaderEntry::Dir(dir) = &self.header {
      dir.file_paths()
    } else {
      unreachable!("asar header is always a directory")
    }
  }

  /// Get a file using the specific path.
  /// 
  /// If the path points to a directory or does not exist, it returns `None`.
  pub async fn get<'a>(&'a mut self, path: impl AsRef<str>) -> Option<File<'a, R>> {
    let path = path.as_ref();
    let segments = path
      .split('/')
      .filter(|x| !x.is_empty())
      .collect::<Vec<_>>();

    let result = self.header.search_segments(&segments);
    if let Some(HeaderEntry::File(metadata)) = result {
      self
        .reader
        .seek(SeekFrom::Start(self.offset + metadata.offset))
        .await
        .unwrap();
      Some(File {
        path: segments.join("/"),
        metadata: &metadata,
        reader: self.reader.borrow_mut().take(metadata.size)
      })
    } else {
      None
    }
  }
}

/// File inside asar archive.
#[derive(Debug)]
pub struct File<'a, R: AsyncRead + Unpin> {
  path: String,
  metadata: &'a FileMetadata,
  reader: Take<&'a mut R>
}

impl<R: AsyncRead + Unpin> File<'_, R> {
  /// The file's name.
  pub fn name(&self) -> &str {
    self.path.split('/').last().unwrap()
  }

  /// The file's path in the archive.
  pub fn path(&self) -> &str {
    &self.path
  }

  /// The size of the file in bytes.
  pub fn size(&self) -> u64 {
    self.metadata.size
  }

  /// Whether the file is executable.
  pub fn executable(&self) -> bool {
    self.metadata.executable
  }

  /// Checksums of the file.
  /// 
  /// Currently manually implementing integrity check for files is needed.
  pub fn integrity(&self) -> Option<&Integrity> {
    self.metadata.integrity.as_ref()
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for File<'_, R> {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut ReadBuf
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut self.reader).poll_read(cx, buf)
  }
}
