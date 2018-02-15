use futures::{Async, AsyncSink, Poll, Sink};
use tokio_io::AsyncWrite;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, Write};

/// A `Write` and `AsyncWrite` implementation for a `Sink` of byte buffers.
#[derive(Debug)]
pub struct SinkWrite<S> {
    sink: S,
}

impl<S: Sink> SinkWrite<S> {
    /// Creates a new `SinkWrite` with the specified `Sink`
    pub fn new(sink: S) -> Self {
        Self { sink: sink }
    }

    /// Unwraps the inner `Sink`.
    pub fn into_inner(self) -> S {
        self.sink
    }
}


impl<S: Sink> Write for SinkWrite<S>
where
    S::SinkItem: for<'a> From<&'a [u8]>,
    IoError: From<S::SinkError>,
{
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        match self.sink.start_send(buf.into())? {
            AsyncSink::NotReady(_) => return Err(IoErrorKind::WouldBlock.into()),
            AsyncSink::Ready => {}
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        async_io!(self.sink.poll_complete()?);

        Ok(())
    }
}

impl<S: Sink> AsyncWrite for SinkWrite<S>
where
    S::SinkItem: for<'a> From<&'a [u8]>,
    IoError: From<S::SinkError>,
{
    fn shutdown(&mut self) -> Poll<(), IoError> {
        Ok(self.sink.poll_complete()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{Future, Sink};
    use std::io::ErrorKind as IoErrorKind;
    use tokio_io::io as async_io;

    #[test]
    fn async_write_works() {
        let sink = Vec::<Vec<u8>>::new().sink_map_err(|_| IoErrorKind::InvalidData);
        let write = SinkWrite::new(sink);

        let (write, _) = async_io::write_all(write, b"hello").wait().unwrap();
        let (write, _) = async_io::write_all(write, b" world").wait().unwrap();
        let write = async_io::shutdown(write).wait().unwrap();

        let vec = write.into_inner().into_inner();

        assert_eq!(&vec[0][..], b"hello");
        assert_eq!(&vec[1][..], b" world");
    }
}
