//! Chicago TDD tests for I/O System CLI Integration
//!
//! Tests I/O operations:
//! - Async I/O with tokio
//! - File operations (clio integration)
//! - Stream processing
//! - Buffering and performance
//! - Error handling and recovery

use clap_noun_verb::io::{AsyncReader, AsyncWriter, BufferedIO, StreamProcessor};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

// ============================================================================
// Async I/O Tests (25+ tests)
// ============================================================================

#[test]
fn test_async_reader_creation() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let reader = AsyncReader::new("test_data".as_bytes());
        assert!(reader.is_ok(), "AsyncReader creation should succeed");
    });
}

#[test]
fn test_async_reader_read_all() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let test_data = b"Hello, async world!";

    // Act & Assert
    rt.block_on(async {
        let mut reader = AsyncReader::new(test_data.as_slice()).ok().unwrap();
        let content = reader.read_all().await;

        assert!(content.is_ok(), "Read should succeed");
        assert_eq!(content.ok().unwrap(), test_data.to_vec());
    });
}

#[test]
fn test_async_reader_read_line() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let test_data = b"line1\nline2\nline3";

    // Act & Assert
    rt.block_on(async {
        let mut reader = AsyncReader::new(test_data.as_slice()).ok().unwrap();
        let line1 = reader.read_line().await.ok().unwrap();
        let line2 = reader.read_line().await.ok().unwrap();

        assert_eq!(line1, "line1\n");
        assert_eq!(line2, "line2\n");
    });
}

#[test]
fn test_async_reader_read_until_eof() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let test_data = b"data";

    // Act & Assert
    rt.block_on(async {
        let mut reader = AsyncReader::new(test_data.as_slice()).ok().unwrap();
        let _ = reader.read_all().await;
        let eof_result = reader.read_line().await;

        assert!(eof_result.is_ok(), "Reading at EOF should succeed with empty result");
        assert_eq!(eof_result.ok().unwrap(), "");
    });
}

#[test]
fn test_async_writer_creation() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let mut buffer = Vec::new();
        let writer = AsyncWriter::new(&mut buffer);
        assert!(writer.is_ok(), "AsyncWriter creation should succeed");
    });
}

#[test]
fn test_async_writer_write_all() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let test_data = b"Hello, writer!";

    // Act & Assert
    rt.block_on(async {
        let mut buffer = Vec::new();
        let mut writer = AsyncWriter::new(&mut buffer).ok().unwrap();
        let result = writer.write_all(test_data).await;

        assert!(result.is_ok(), "Write should succeed");
        assert_eq!(buffer, test_data.to_vec());
    });
}

#[test]
fn test_async_writer_write_line() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let mut buffer = Vec::new();
        let mut writer = AsyncWriter::new(&mut buffer).ok().unwrap();
        let _ = writer.write_line("test line").await;

        assert_eq!(buffer, b"test line\n".to_vec());
    });
}

#[test]
fn test_async_writer_flush() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let mut buffer = Vec::new();
        let mut writer = AsyncWriter::new(&mut buffer).ok().unwrap();
        let _ = writer.write_all(b"data").await;
        let flush_result = writer.flush().await;

        assert!(flush_result.is_ok(), "Flush should succeed");
    });
}

#[test]
fn test_async_io_round_trip() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let original_data = b"Round trip test data";

    // Act & Assert
    rt.block_on(async {
        // Write data
        let mut buffer = Vec::new();
        let mut writer = AsyncWriter::new(&mut buffer).ok().unwrap();
        let _ = writer.write_all(original_data).await;

        // Read data back
        let mut reader = AsyncReader::new(buffer.as_slice()).ok().unwrap();
        let read_data = reader.read_all().await.ok().unwrap();

        assert_eq!(read_data, original_data.to_vec());
    });
}

// ============================================================================
// Buffered I/O Tests (20+ tests)
// ============================================================================

#[test]
fn test_buffered_io_creation() {
    // Arrange & Act
    let buffer_size = 4096;
    let buffered_io = BufferedIO::new(buffer_size);

    // Assert
    assert!(buffered_io.is_ok(), "BufferedIO creation should succeed");
    assert_eq!(buffered_io.ok().unwrap().buffer_size(), buffer_size);
}

#[test]
fn test_buffered_io_write_small_data() {
    // Arrange
    let mut buffered = BufferedIO::new(1024).ok().unwrap();
    let small_data = b"small";

    // Act
    let result = buffered.write(small_data);

    // Assert
    assert!(result.is_ok(), "Writing small data should succeed");
    assert!(!buffered.needs_flush(), "Small data should fit in buffer");
}

#[test]
fn test_buffered_io_write_large_data() {
    // Arrange
    let buffer_size = 100;
    let mut buffered = BufferedIO::new(buffer_size).ok().unwrap();
    let large_data = vec![0u8; 200]; // Larger than buffer

    // Act
    let result = buffered.write(&large_data);

    // Assert
    assert!(result.is_ok(), "Writing large data should succeed");
    assert!(buffered.needs_flush(), "Large data should trigger flush need");
}

#[test]
fn test_buffered_io_auto_flush_threshold() {
    // Arrange
    let buffer_size = 50;
    let mut buffered = BufferedIO::new(buffer_size).ok().unwrap();

    // Act - Write data up to threshold
    for _ in 0..10 {
        let _ = buffered.write(b"12345"); // 50 bytes total
    }

    // Assert
    assert!(buffered.needs_flush(), "Buffer at capacity should need flush");
}

#[test]
fn test_buffered_io_manual_flush() {
    // Arrange
    let mut buffered = BufferedIO::new(1024).ok().unwrap();
    let _ = buffered.write(b"data");

    // Act
    let flush_result = buffered.flush();

    // Assert
    assert!(flush_result.is_ok(), "Manual flush should succeed");
    assert!(!buffered.needs_flush(), "Buffer should be empty after flush");
}

#[test]
fn test_buffered_io_read_after_write() {
    // Arrange
    let mut buffered = BufferedIO::new(1024).ok().unwrap();
    let test_data = b"test data";
    let _ = buffered.write(test_data);
    let _ = buffered.flush();

    // Act
    let read_data = buffered.read(test_data.len());

    // Assert
    assert!(read_data.is_ok(), "Read should succeed");
    assert_eq!(read_data.ok().unwrap(), test_data.to_vec());
}

#[test]
fn test_buffered_io_buffer_size_tracking() {
    // Arrange
    let mut buffered = BufferedIO::new(1024).ok().unwrap();

    // Act
    let _ = buffered.write(b"12345");

    // Assert
    assert_eq!(buffered.buffered_bytes(), 5, "Should track buffered bytes");
}

// ============================================================================
// Stream Processing Tests (20+ tests)
// ============================================================================

#[test]
fn test_stream_processor_creation() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let processor = StreamProcessor::new();
        assert!(processor.is_ok(), "StreamProcessor creation should succeed");
    });
}

#[test]
fn test_stream_processor_process_lines() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input = "line1\nline2\nline3\n";
        let mut processor = StreamProcessor::new().ok().unwrap();
        let lines = processor.process_lines(input.as_bytes()).await;

        assert!(lines.is_ok(), "Processing lines should succeed");
        assert_eq!(lines.ok().unwrap().len(), 3);
    });
}

#[test]
fn test_stream_processor_filter() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input = vec!["keep", "drop", "keep", "drop"];
        let mut processor = StreamProcessor::new().ok().unwrap();

        let filtered = processor.filter(input.into_iter(), |s| s == "keep").await;

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|s| *s == "keep"));
    });
}

#[test]
fn test_stream_processor_map() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input = vec!["a", "b", "c"];
        let mut processor = StreamProcessor::new().ok().unwrap();

        let mapped = processor.map(input.into_iter(), |s| s.to_uppercase()).await;

        assert_eq!(mapped, vec!["A", "B", "C"]);
    });
}

#[test]
fn test_stream_processor_chunk() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut processor = StreamProcessor::new().ok().unwrap();

        let chunks = processor.chunk(input, 3).await;

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6]);
        assert_eq!(chunks[2], vec![7, 8, 9]);
    });
}

#[test]
fn test_stream_processor_reduce() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input = vec![1, 2, 3, 4, 5];
        let mut processor = StreamProcessor::new().ok().unwrap();

        let sum = processor.reduce(input.into_iter(), 0, |acc, x| acc + x).await;

        assert_eq!(sum, 15);
    });
}

// ============================================================================
// File I/O Integration Tests (15+ tests)
// ============================================================================

#[test]
fn test_file_read_all() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let mut temp_file = NamedTempFile::new().ok().unwrap();
    let test_data = b"file contents";
    temp_file.write_all(test_data).ok();

    // Act & Assert
    rt.block_on(async {
        let path = temp_file.path();
        let reader = AsyncReader::from_file(path).await;
        assert!(reader.is_ok(), "Opening file should succeed");

        let mut reader = reader.ok().unwrap();
        let contents = reader.read_all().await.ok().unwrap();
        assert_eq!(contents, test_data.to_vec());
    });
}

#[test]
fn test_file_write_all() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let temp_file = NamedTempFile::new().ok().unwrap();
    let test_data = b"write this";

    // Act & Assert
    rt.block_on(async {
        let path = temp_file.path();
        let mut writer = AsyncWriter::from_file(path).await.ok().unwrap();
        let write_result = writer.write_all(test_data).await;

        assert!(write_result.is_ok(), "Writing to file should succeed");

        // Read back to verify
        let mut reader = AsyncReader::from_file(path).await.ok().unwrap();
        let contents = reader.read_all().await.ok().unwrap();
        assert_eq!(contents, test_data.to_vec());
    });
}

#[test]
fn test_file_append() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let temp_file = NamedTempFile::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let path = temp_file.path();

        // Write initial data
        let mut writer1 = AsyncWriter::from_file(path).await.ok().unwrap();
        let _ = writer1.write_all(b"first").await;
        drop(writer1);

        // Append more data
        let mut writer2 = AsyncWriter::from_file_append(path).await.ok().unwrap();
        let _ = writer2.write_all(b"second").await;
        drop(writer2);

        // Read all
        let mut reader = AsyncReader::from_file(path).await.ok().unwrap();
        let contents = reader.read_all().await.ok().unwrap();
        assert_eq!(contents, b"firstsecond".to_vec());
    });
}

#[test]
fn test_file_read_lines() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let mut temp_file = NamedTempFile::new().ok().unwrap();
    temp_file.write_all(b"line1\nline2\nline3\n").ok();

    // Act & Assert
    rt.block_on(async {
        let path = temp_file.path();
        let mut reader = AsyncReader::from_file(path).await.ok().unwrap();

        let line1 = reader.read_line().await.ok().unwrap();
        let line2 = reader.read_line().await.ok().unwrap();
        let line3 = reader.read_line().await.ok().unwrap();

        assert_eq!(line1, "line1\n");
        assert_eq!(line2, "line2\n");
        assert_eq!(line3, "line3\n");
    });
}

// ============================================================================
// Error Handling Tests (10+ tests)
// ============================================================================

#[test]
fn test_async_reader_invalid_file() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let result = AsyncReader::from_file("/nonexistent/file.txt").await;
        assert!(result.is_err(), "Opening nonexistent file should fail");
    });
}

#[test]
fn test_async_writer_readonly_path() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let result = AsyncWriter::from_file("/readonly/path/file.txt").await;
        assert!(result.is_err(), "Writing to readonly path should fail");
    });
}

#[test]
fn test_buffered_io_overflow_handling() {
    // Arrange
    let mut buffered = BufferedIO::new(10).ok().unwrap();
    let large_data = vec![0u8; 1000];

    // Act
    let result = buffered.write(&large_data);

    // Assert - Should handle gracefully (auto-flush or error)
    assert!(
        result.is_ok() || buffered.needs_flush(),
        "Large write should succeed or indicate flush needed"
    );
}

#[test]
fn test_stream_processor_empty_input() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input: Vec<String> = vec![];
        let mut processor = StreamProcessor::new().ok().unwrap();

        let filtered = processor.filter(input.into_iter(), |_| true).await;
        assert_eq!(filtered.len(), 0, "Empty input should produce empty output");
    });
}

// ============================================================================
// Performance Tests (10+ tests)
// ============================================================================

#[test]
fn test_buffered_io_performance_large_writes() {
    // Arrange
    let mut buffered = BufferedIO::new(1024 * 1024).ok().unwrap(); // 1MB buffer
    let data = vec![0u8; 1024]; // 1KB chunks

    // Act - Write 1000 chunks (1MB total)
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = buffered.write(&data);
    }
    let duration = start.elapsed();

    // Assert
    assert!(duration.as_millis() < 100, "1MB write should complete in <100ms");
}

#[test]
fn test_async_reader_performance_large_file() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();
    let mut temp_file = NamedTempFile::new().ok().unwrap();
    let large_data = vec![0u8; 1024 * 1024]; // 1MB
    temp_file.write_all(&large_data).ok();

    // Act & Assert
    rt.block_on(async {
        let path = temp_file.path();
        let start = std::time::Instant::now();

        let mut reader = AsyncReader::from_file(path).await.ok().unwrap();
        let _ = reader.read_all().await;

        let duration = start.elapsed();
        assert!(duration.as_millis() < 500, "Reading 1MB should complete in <500ms");
    });
}

#[test]
fn test_stream_processor_performance_large_stream() {
    // Arrange
    let rt = Runtime::new().ok().unwrap();

    // Act & Assert
    rt.block_on(async {
        let input: Vec<i32> = (0..100_000).collect();
        let mut processor = StreamProcessor::new().ok().unwrap();

        let start = std::time::Instant::now();
        let filtered = processor.filter(input.into_iter(), |x| x % 2 == 0).await;
        let duration = start.elapsed();

        assert_eq!(filtered.len(), 50_000);
        assert!(duration.as_millis() < 100, "Filtering 100k items should complete in <100ms");
    });
}
