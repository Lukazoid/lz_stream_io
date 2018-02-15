//! This crate provides `AsyncRead` and `AsyncWrite` implementations for
//! `Stream` and `Sink` instances over byte buffers.
//! 
//! The `StreamRead` and `SinkWrite` types may be used like so:
//! 
//! ```
//! extern crate bytes;
//! extern crate futures;
//! extern crate tokio_io;
//! extern crate lz_stream_io;
//! 
//! use futures::{Future, Stream, Sink};
//! use futures::unsync::mpsc;
//! use std::io::{Result as IoResult, ErrorKind as IoErrorKind};
//! use bytes::Bytes;
//! use tokio_io::io as async_io;
//! use lz_stream_io::{SinkWrite, StreamRead};
//! 
//! fn main() {
//!     // The sink item type must implement From<&[u8]>
//!     // The stream item type must implement AsRef<[u8]>
//!     let (sink, stream) = mpsc::unbounded::<Bytes>();
//! 
//!     // Both sink and stream must have an error type which std::io::Error
//!     // can be created from
//!     let sink = sink.sink_map_err(|_| IoErrorKind::InvalidData);
//!     let stream = stream.map_err(|_| IoErrorKind::InvalidData);
//! 
//!     let write = SinkWrite::new(sink);
//!     let read = StreamRead::new(stream);
//! 
//!     async_io::write_all(write, b"hello")
//!         .and_then(|(write, _)| async_io::write_all(write, b" world"))
//!         .and_then(|_| async_io::read_to_end(read, vec![]))
//!         .and_then(|(_, bytes)| {
//!             assert_eq!(bytes, b"hello world");
//!             Ok(())
//!         }).wait().unwrap();
//! }
//! 
//! ```

extern crate bytes;
extern crate futures;
extern crate tokio_io;

macro_rules! async_io {
    ($e: expr) => (match $e {
        ::futures::Async::Ready(result) => result,
        ::futures::Async::NotReady => return Err(::std::io::ErrorKind::WouldBlock.into()),
    })
}

mod stream_read;
pub use stream_read::StreamRead;

mod sink_write;
pub use sink_write::SinkWrite;