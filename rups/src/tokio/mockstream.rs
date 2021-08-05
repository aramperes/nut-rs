use std::fmt;
use std::io::{Error, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Async stream for unit testing.
#[derive(Clone, Default)]
pub struct AsyncMockStream(mockstream::SyncMockStream);

impl AsyncMockStream {
    /// Create empty stream
    pub fn new() -> AsyncMockStream {
        AsyncMockStream::default()
    }

    /// Extract all bytes written by Write trait calls.
    pub fn push_bytes_to_read(&mut self, bytes: &[u8]) {
        self.0.push_bytes_to_read(bytes)
    }
}

impl fmt::Debug for AsyncMockStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncMockStream").finish()
    }
}

impl Read for AsyncMockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for AsyncMockStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl AsyncRead for AsyncMockStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut vec = Vec::new();
        match self.get_mut().read_to_end(&mut vec) {
            Ok(_) => {
                let slice = vec.as_slice();
                buf.put_slice(slice);
                Poll::Ready(Ok(()))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for AsyncMockStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let len = buf.len();
        self.get_mut().push_bytes_to_read(buf);
        Poll::Ready(Ok(len))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(self.get_mut().flush())
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}
