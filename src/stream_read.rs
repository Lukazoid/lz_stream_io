use futures::{Async, Stream};
use futures::stream::Fuse;
use tokio_io::AsyncRead;
use std::io::{BufRead, Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult};

/// A `Read`, `BufRead` and `AsyncRead` implementation for a `Stream` of byte
/// buffers.
#[derive(Debug)]
pub struct StreamRead<S> {
    stream: Fuse<S>,
    buf: Vec<u8>,
}

impl<S: Stream> StreamRead<S> {
    /// Creates a new `StreamRead` with the specified `Stream`
    pub fn new(stream: S) -> Self {
        Self {
            stream: stream.fuse(),
            buf: Vec::default(),
        }
    }

    /// Unwraps the inner `Stream`.
    pub fn into_inner(self) -> S {
        self.stream.into_inner()
    }
}

impl<S: Stream> Read for StreamRead<S>
where
    S::Item: AsRef<[u8]>,
    IoError: From<S::Error>,
{
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let n = self.fill_buf()?.read(buf)?;

        self.consume(n);

        Ok(n)
    }
}

impl<S: Stream> BufRead for StreamRead<S>
where
    S::Item: AsRef<[u8]>,
    IoError: From<S::Error>,
{
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        if self.buf.is_empty() {
            match self.stream.poll()? {
                Async::Ready(Some(item)) => {
                    self.buf.extend_from_slice(item.as_ref());
                }
                Async::Ready(None) => {}
                Async::NotReady => return Err(IoErrorKind::WouldBlock.into()),
            }
        }
        Ok(&self.buf[..])
    }

    fn consume(&mut self, amt: usize) {
        self.buf = self.buf.split_off(amt);
    }
}

impl<S: Stream> AsyncRead for StreamRead<S>
where
    S::Item: AsRef<[u8]>,
    IoError: From<S::Error>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use futures::sync::mpsc;
    use std::io::ErrorKind as IoErrorKind;
    use std::thread;
    use std::time::Duration;
    use tokio_io::io as async_io;

    #[test]
    fn async_read_works() {
        let (tx, rx) = mpsc::unbounded();

        let sender_thread = thread::spawn(move || {
            tx.unbounded_send(&b"he"[..]).unwrap();
            thread::sleep(Duration::from_millis(20));
            tx.unbounded_send(&b"llo"[..]).unwrap();
            thread::sleep(Duration::from_millis(20));
            tx.unbounded_send(&b" wor"[..]).unwrap();
            thread::sleep(Duration::from_millis(20));
            tx.unbounded_send(&b"ld"[..]).unwrap();
        });

        let read = StreamRead::new(rx.map_err(|_| IoErrorKind::InvalidData));

        let (_, output) = async_io::read_to_end(read, vec![]).wait().unwrap();

        assert_eq!(output, b"hello world");

        sender_thread.join().unwrap();
    }
}
