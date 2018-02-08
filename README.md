# Lz Stream IO

This crate provides IO abstractions over the `futures::Stream` and `futures::Sink` types.

[![Build Status](https://travis-ci.org/Lukazoid/lz_stream_io.svg?branch=master)](https://travis-ci.org/Lukazoid/lz_stream_io)

[Documentation](https://docs.rs/lz_stream_io)

## Features 
 - A `std::io::Write` and `tokio_io::AsyncWrite` implementation over a `futures::Sink` (see `lz_stream_io::SinkWrite`).
 - A `std::io::Read` and `tokio_io::Async` implementation over a `futures::Stream` (see `lz_stream_io::StreamRead`).

# License

This project is licensed under the MIT License ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).