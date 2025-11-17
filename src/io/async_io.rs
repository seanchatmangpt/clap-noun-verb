//! Asynchronous I/O Support with Tokio
//!
//! Provides async-native traits and utilities for building high-performance CLI applications
//! with Tokio. Features include:
//!
//! - Async Input/Output traits extending tokio::io
//! - Backpressure handling with configurable buffers
//! - Framed I/O with protocol support
//! - TCP and Unix socket helpers
//! - Bidirectional stream support
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::io::async_io::{AsyncInputExt, BackpressureConfig};
//! use tokio::net::TcpStream;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
//!     let data = stream.read_all_async().await?;
//!     Ok(())
//! }
//! ```

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

/// Configuration for backpressure handling
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum buffer size before applying backpressure (bytes)
    pub max_buffer_size: usize,
    /// Number of bytes to read per iteration
    pub chunk_size: usize,
    /// Enable adaptive buffer sizing
    pub adaptive: bool,
}

impl BackpressureConfig {
    /// Create with default settings (64KB buffer, 8KB chunks)
    pub fn new() -> Self {
        Self {
            max_buffer_size: 64 * 1024,
            chunk_size: 8 * 1024,
            adaptive: true,
        }
    }

    /// Set maximum buffer size
    pub fn with_max_buffer(mut self, size: usize) -> Self {
        self.max_buffer_size = size;
        self
    }

    /// Set chunk size for reads
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Enable/disable adaptive buffering
    pub fn with_adaptive(mut self, adaptive: bool) -> Self {
        self.adaptive = adaptive;
        self
    }
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Async input trait extending tokio::io::AsyncRead
pub trait AsyncInputExt: AsyncRead + Unpin {
    /// Read all available data into a Vec
    async fn read_all_async(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    /// Read all available data into a String
    async fn read_string_async(&mut self) -> std::io::Result<String> {
        let mut string = String::new();
        self.read_to_string(&mut string).await?;
        Ok(string)
    }

    /// Read with backpressure handling
    async fn read_with_backpressure(
        &mut self,
        config: &BackpressureConfig,
    ) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut chunk = vec![0u8; config.chunk_size];

        loop {
            // Apply backpressure if buffer is too large
            if buffer.len() >= config.max_buffer_size {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            match self.read(&mut chunk).await? {
                0 => break,
                n => buffer.extend_from_slice(&chunk[..n]),
            }
        }

        Ok(buffer)
    }

    /// Read exact number of bytes
    async fn read_exact_async(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer).await?;
        Ok(buffer)
    }
}

// Implement for all types that implement AsyncRead
impl<T: AsyncRead + Unpin + ?Sized> AsyncInputExt for T {}

/// Async output trait extending tokio::io::AsyncWrite
pub trait AsyncOutputExt: AsyncWrite + Unpin {
    /// Write all data and flush
    async fn write_all_async(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write_all(buf).await?;
        self.flush().await?;
        Ok(())
    }

    /// Write string and flush
    async fn write_string_async(&mut self, s: &str) -> std::io::Result<()> {
        self.write_all_async(s.as_bytes()).await
    }

    /// Write with backpressure handling
    async fn write_with_backpressure(
        &mut self,
        data: &[u8],
        config: &BackpressureConfig,
    ) -> std::io::Result<()> {
        let mut written = 0;

        while written < data.len() {
            let chunk_size = std::cmp::min(config.chunk_size, data.len() - written);
            self.write_all(&data[written..written + chunk_size]).await?;
            self.flush().await?;
            written += chunk_size;

            // Yield to allow other tasks to run
            if written < data.len() {
                tokio::task::yield_now().await;
            }
        }

        Ok(())
    }

    /// Write formatted output
    async fn write_fmt_async(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        use std::fmt::Write;
        let mut buffer = String::new();
        write!(&mut buffer, "{}", args).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "format error")
        })?;
        self.write_all_async(buffer.as_bytes()).await
    }
}

// Implement for all types that implement AsyncWrite
impl<T: AsyncWrite + Unpin + ?Sized> AsyncOutputExt for T {}

/// Bidirectional stream abstraction
pub struct BidirectionalStream<T: AsyncRead + AsyncWrite + Unpin> {
    inner: T,
    read_buffer: BytesMut,
    #[allow(dead_code)]
    write_buffer: BytesMut,
    backpressure_config: BackpressureConfig,
}

impl<T: AsyncRead + AsyncWrite + Unpin> BidirectionalStream<T> {
    /// Create new bidirectional stream
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            read_buffer: BytesMut::with_capacity(8192),
            write_buffer: BytesMut::with_capacity(8192),
            backpressure_config: BackpressureConfig::new(),
        }
    }

    /// Create with custom backpressure config
    pub fn with_config(inner: T, config: BackpressureConfig) -> Self {
        Self {
            inner,
            read_buffer: BytesMut::with_capacity(config.chunk_size),
            write_buffer: BytesMut::with_capacity(config.chunk_size),
            backpressure_config: config,
        }
    }

    /// Read from stream with buffering
    pub async fn read_frame(&mut self) -> std::io::Result<Option<BytesMut>> {
        let mut buf = vec![0u8; self.backpressure_config.chunk_size];
        match self.inner.read(&mut buf).await? {
            0 => Ok(None),
            n => {
                self.read_buffer.extend_from_slice(&buf[..n]);
                Ok(Some(self.read_buffer.split()))
            }
        }
    }

    /// Write to stream with buffering
    pub async fn write_frame(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(data).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// Get backpressure config
    pub fn backpressure_config(&self) -> &BackpressureConfig {
        &self.backpressure_config
    }

    /// Set backpressure config
    pub fn set_backpressure_config(&mut self, config: BackpressureConfig) {
        self.backpressure_config = config;
    }

    /// Consume and return inner stream
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get mutable reference to inner stream
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

/// Protocol frame builder for length-delimited messages
pub struct LengthDelimitedFrameBuilder {
    pub(crate) max_frame_size: u32,
}

impl LengthDelimitedFrameBuilder {
    /// Create new builder with default max size (1MB)
    pub fn new() -> Self {
        Self {
            max_frame_size: 1024 * 1024,
        }
    }

    /// Set maximum frame size
    pub fn with_max_size(mut self, size: u32) -> Self {
        self.max_frame_size = size;
        self
    }

    /// Get maximum frame size
    pub fn max_frame_size(&self) -> u32 {
        self.max_frame_size
    }

    /// Build a frame with length prefix
    pub fn build(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        if data.len() > self.max_frame_size as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Frame too large: {} > {}", data.len(), self.max_frame_size),
            ));
        }

        let len = data.len() as u32;
        let mut frame = len.to_le_bytes().to_vec();
        frame.extend_from_slice(data);
        Ok(frame)
    }

    /// Parse length-delimited frame
    pub fn parse(&self, frame: &[u8]) -> Result<Option<(u32, usize)>, std::io::Error> {
        if frame.len() < 4 {
            return Ok(None);
        }

        let len_bytes: [u8; 4] = frame[..4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes);

        if len > self.max_frame_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame size exceeds maximum: {}", len),
            ));
        }

        if frame.len() < (4 + len as usize) {
            return Ok(Some((len, 4)));
        }

        Ok(Some((len, frame.len())))
    }
}

impl Default for LengthDelimitedFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Lines-based frame builder for line-delimited messages
pub struct LinesFrameBuilder {
    pub(crate) max_line_size: usize,
}

impl LinesFrameBuilder {
    /// Create new builder with default max size (64KB per line)
    pub fn new() -> Self {
        Self {
            max_line_size: 64 * 1024,
        }
    }

    /// Set maximum line size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_line_size = size;
        self
    }

    /// Get maximum line size
    pub fn max_line_size(&self) -> usize {
        self.max_line_size
    }

    /// Build a line frame
    pub fn build(&self, data: &str) -> Result<Vec<u8>, std::io::Error> {
        let bytes = data.as_bytes();
        if bytes.len() > self.max_line_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Line too large: {} > {}", bytes.len(), self.max_line_size),
            ));
        }

        let mut frame = bytes.to_vec();
        frame.push(b'\n');
        Ok(frame)
    }

    /// Parse line frame (expects data ending with \n or EOF)
    pub fn parse(&self, data: &[u8]) -> Option<usize> {
        data.windows(1)
            .position(|w| w[0] == b'\n')
            .map(|pos| pos + 1)
    }
}

impl Default for LinesFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_config() {
        let config = BackpressureConfig::new()
            .with_max_buffer(256 * 1024)
            .with_chunk_size(16 * 1024)
            .with_adaptive(false);

        assert_eq!(config.max_buffer_size, 256 * 1024);
        assert_eq!(config.chunk_size, 16 * 1024);
        assert!(!config.adaptive);
    }

    #[test]
    fn test_length_delimited_frame() {
        let builder = LengthDelimitedFrameBuilder::new();
        let data = b"Hello, World!";
        let frame = builder.build(data).unwrap();

        assert_eq!(frame.len(), 4 + data.len());
        assert_eq!(&frame[4..], data);
    }

    #[test]
    fn test_length_delimited_parse() {
        let builder = LengthDelimitedFrameBuilder::new();
        let data = vec![13, 0, 0, 0, b'H', b'e', b'l', b'l', b'o'];

        let result = builder.parse(&data).unwrap().unwrap();
        assert_eq!(result.0, 13);
    }

    #[test]
    fn test_lines_frame() {
        let builder = LinesFrameBuilder::new();
        let line = "test line";
        let frame = builder.build(line).unwrap();

        assert!(frame.ends_with(b"\n"));
        assert_eq!(&frame[..frame.len() - 1], line.as_bytes());
    }

    #[test]
    fn test_lines_frame_parse() {
        let builder = LinesFrameBuilder::new();
        let data = b"test line\nrest";

        let pos = builder.parse(data).unwrap();
        assert_eq!(pos, 10);
    }

    #[tokio::test]
    async fn test_async_input_ext() {
        use std::io::Cursor;

        let data = b"Hello, World!";
        let cursor = Cursor::new(data);
        let mut async_reader = tokio::io::BufReader::new(cursor);

        let result = async_reader.read_string_async().await.unwrap();
        assert_eq!(result, "Hello, World!");
    }
}
