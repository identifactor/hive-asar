use std::borrow::BorrowMut;
use std::io::SeekFrom;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, ReadBuf, Take};
use crate::header::{Entry, FileMetadata, Header};

pub struct Archive<R: AsyncRead + AsyncSeek + Unpin> {
  offset: u64,
  header: Header,
  reader: R
}

impl<R: AsyncRead + AsyncSeek + Unpin> Archive<R> {
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

  pub async fn get_file<'a>(&'a mut self, path: impl AsRef<str>) -> Option<File<'a, R>> {
    let segments = path
      .as_ref()
      .split('/')
      .filter(|x| !x.is_empty())
      .collect::<Vec<_>>();
    let result = self.header.search_segments(&segments);
    if let Some(Entry::File(metadata)) = result {
      self.reader.seek(SeekFrom::Start(self.offset + metadata.offset)).await.unwrap();
      Some(File {
        metadata: &metadata,
        reader: self.reader.borrow_mut().take(metadata.size)
      })
    } else {
      None
    }
  }
}

pub struct File<'a, R: AsyncRead + Unpin> {
  metadata: &'a FileMetadata,
  reader: Take<&'a mut R>
}

impl<R: AsyncRead + Unpin> File<'_, R> {
  pub fn metadata(&self) -> &FileMetadata {
    self.metadata
  }
}

impl<R: AsyncRead + Unpin> AsyncRead for File<'_, R> {
  fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut ReadBuf) -> Poll<io::Result<()>> {
    Pin::new(&mut self.reader).poll_read(cx, buf)
  }
}

#[cfg(test)]
#[tokio::test]
async fn test() -> io::Result<()> {
  let mut a = Archive::new(tokio::fs::File::open("a.asar").await?).await?;
  let mut src = a.get_file("lib.rs").await.unwrap();
  let mut code = String::with_capacity(src.metadata.size as _);
  src.read_to_string(&mut code).await?;
  println!("{}", code);
  Ok(())
}
