//! Async I/O Integration Tests
//!
//! Comprehensive tests for Tokio-based async I/O capabilities

#[cfg(test)]
mod async_io_tests {
    use clap_noun_verb::io::async_io::{
        AsyncInputExt, AsyncOutputExt, BackpressureConfig, LengthDelimitedFrameBuilder,
        LinesFrameBuilder,
    };
    use std::io::Cursor;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_backpressure_config_builder() {
        let config = BackpressureConfig::new()
            .with_max_buffer(256 * 1024)
            .with_chunk_size(16 * 1024)
            .with_adaptive(true);

        assert_eq!(config.max_buffer_size, 256 * 1024);
        assert_eq!(config.chunk_size, 16 * 1024);
        assert!(config.adaptive);
    }

    #[test]
    fn test_backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.max_buffer_size, 64 * 1024);
        assert_eq!(config.chunk_size, 8 * 1024);
        assert!(config.adaptive);
    }

    #[test]
    fn test_backpressure_config_clone() {
        let config1 = BackpressureConfig::new().with_max_buffer(128 * 1024);
        let config2 = config1.clone();

        assert_eq!(config1.max_buffer_size, config2.max_buffer_size);
    }

    #[tokio::test]
    async fn test_async_read_all() {
        let data = b"Hello, async world!";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = tokio::io::BufReader::new(cursor);

        let result = reader.read_all_async().await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_async_read_string() {
        let data = "Test string";
        let cursor = Cursor::new(data.to_string().into_bytes());
        let mut reader = tokio::io::BufReader::new(cursor);

        let result = reader.read_string_async().await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_async_read_exact() {
        let data = b"0123456789";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = tokio::io::BufReader::new(cursor);

        let result = reader.read_exact_async(5).await.unwrap();
        assert_eq!(result, &data[..5]);
    }

    #[tokio::test]
    async fn test_async_write_all() {
        let mut buffer = Vec::new();
        buffer.write_all_async(b"test data").await.unwrap();
        assert_eq!(buffer, b"test data");
    }

    #[tokio::test]
    async fn test_async_write_string() {
        let mut buffer = Vec::new();
        buffer.write_string_async("test string").await.unwrap();
        assert_eq!(buffer, b"test string");
    }

    #[tokio::test]
    async fn test_length_delimited_frame_builder() {
        let builder = LengthDelimitedFrameBuilder::new();
        let data = b"Hello, World!";

        let frame = builder.build(data).unwrap();
        assert_eq!(frame.len(), 4 + data.len());

        // First 4 bytes should be length in little-endian
        let len_bytes: [u8; 4] = frame[..4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes);
        assert_eq!(len as usize, data.len());

        // Rest should be data
        assert_eq!(&frame[4..], data);
    }

    #[test]
    fn test_length_delimited_max_size() {
        let builder = LengthDelimitedFrameBuilder::new().with_max_size(10);
        let data = [0u8; 20];

        let result = builder.build(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_length_delimited_parse_incomplete() {
        let builder = LengthDelimitedFrameBuilder::new();
        let data = vec![13, 0, 0, 0, b'H', b'e'];

        let result = builder.parse(&data).unwrap();
        assert!(result.is_some());
        let (len, _pos) = result.unwrap();
        assert_eq!(len, 13);
        // Data is incomplete, so position points to header
        assert_eq!(data.len(), 6);
    }

    #[test]
    fn test_length_delimited_parse_complete() {
        let builder = LengthDelimitedFrameBuilder::new();
        let data = vec![
            5, 0, 0, 0, // length = 5
            b'H', b'e', b'l', b'l', b'o', // data
        ];

        let result = builder.parse(&data).unwrap();
        assert!(result.is_some());
        let (len, pos) = result.unwrap();
        assert_eq!(len, 5);
        assert_eq!(pos, data.len());
    }

    #[test]
    fn test_lines_frame_builder() {
        let builder = LinesFrameBuilder::new();
        let line = "test line";

        let frame = builder.build(line).unwrap();
        assert_eq!(frame.last(), Some(&b'\n'));
        assert_eq!(&frame[..frame.len() - 1], line.as_bytes());
    }

    #[test]
    fn test_lines_frame_max_size() {
        let builder = LinesFrameBuilder::new().with_max_size(5);
        let long_line = "this is a long line";

        let result = builder.build(long_line);
        assert!(result.is_err());
    }

    #[test]
    fn test_lines_frame_parse() {
        let builder = LinesFrameBuilder::new();
        let data = b"line 1\nline 2\nrest";

        let pos1 = builder.parse(&data[..7]).unwrap();
        assert_eq!(pos1, 7); // Position after first \n

        let pos2 = builder.parse(&data[7..14]).unwrap();
        assert_eq!(pos2, 7); // Position after second \n
    }

    #[test]
    fn test_lines_frame_parse_no_newline() {
        let builder = LinesFrameBuilder::new();
        let data = b"line without newline";

        let result = builder.parse(data);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_write_with_backpressure() {
        let mut buffer = Vec::new();
        let data = b"test data for backpressure";
        let config = BackpressureConfig::new().with_chunk_size(8);

        buffer.write_with_backpressure(data, &config).await.unwrap();

        assert_eq!(buffer, data);
    }

    #[test]
    fn test_length_delimited_default() {
        let builder = LengthDelimitedFrameBuilder::default();
        assert_eq!(builder.max_frame_size(), 1024 * 1024);
    }

    #[test]
    fn test_lines_frame_default() {
        let builder = LinesFrameBuilder::default();
        assert_eq!(builder.max_line_size(), 64 * 1024);
    }

    #[tokio::test]
    async fn test_async_write_fmt() {
        let mut buffer = Vec::new();
        buffer.write_fmt_async(format_args!("{} {}", "hello", "world")).await.unwrap();

        assert_eq!(buffer, b"hello world");
    }

    #[test]
    fn test_bidirectional_stream_creation() {
        use clap_noun_verb::io::async_io::BidirectionalStream;

        let data = b"test data";
        let cursor = Cursor::new(data.to_vec());
        let stream = BidirectionalStream::new(cursor);

        assert_eq!(stream.backpressure_config().max_buffer_size, 64 * 1024);
    }

    #[test]
    fn test_bidirectional_stream_with_config() {
        use clap_noun_verb::io::async_io::BidirectionalStream;

        let data = b"test data";
        let cursor = Cursor::new(data.to_vec());
        let config = BackpressureConfig::new().with_max_buffer(128 * 1024);
        let stream = BidirectionalStream::with_config(cursor, config);

        assert_eq!(stream.backpressure_config().max_buffer_size, 128 * 1024);
    }

    #[test]
    fn test_length_delimited_frame_size_exceeded() {
        let builder = LengthDelimitedFrameBuilder::new().with_max_size(100);
        let large_data = vec![0u8; 1000];

        let result = builder.build(&large_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_lines_frame_builder_builder_pattern() {
        let builder = LinesFrameBuilder::new().with_max_size(128 * 1024).with_max_size(256 * 1024);

        assert_eq!(builder.max_line_size(), 256 * 1024);
    }

    #[test]
    fn test_backpressure_config_adaptive_flag() {
        let config1 = BackpressureConfig::new().with_adaptive(true);
        let config2 = BackpressureConfig::new().with_adaptive(false);

        assert!(config1.adaptive);
        assert!(!config2.adaptive);
    }
}

#[cfg(test)]
mod async_stress_tests {
    use clap_noun_verb::io::async_io::{AsyncOutputExt, BackpressureConfig};

    #[tokio::test]
    async fn test_high_throughput_writes() {
        let mut buffer = Vec::new();
        let config = BackpressureConfig::new().with_chunk_size(1024);

        // Write 10MB of data in chunks
        for _ in 0..1000 {
            let data = vec![42u8; 10_000];
            buffer.write_with_backpressure(&data, &config).await.unwrap();
        }

        assert_eq!(buffer.len(), 10_000_000);
    }

    #[tokio::test]
    async fn test_concurrent_async_reads() {
        use clap_noun_verb::io::async_io::AsyncInputExt;
        use std::io::Cursor;

        let data = b"test data for concurrent reads";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = tokio::io::BufReader::new(cursor);

        let result = reader.read_all_async().await.unwrap();
        assert_eq!(result, data);
    }
}
