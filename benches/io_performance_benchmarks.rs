use clap_noun_verb::io::{IoPipeline, IoPipelineBuilder};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::{Read, Write};
use std::time::Duration;

/// Benchmark I/O pipeline construction
fn bench_io_pipeline_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_pipeline_construction");

    group.bench_function("pipeline_builder_default", |b| {
        b.iter(|| {
            black_box(IoPipelineBuilder::new().build());
        });
    });

    group.bench_function("pipeline_with_buffer_config", |b| {
        b.iter(|| {
            black_box(IoPipelineBuilder::new().buffer_size(16384).build());
        });
    });

    group.bench_function("pipeline_direct_construction", |b| {
        b.iter(|| {
            black_box(IoPipeline::new());
        });
    });

    group.finish();
}

/// Benchmark I/O buffer operations
fn bench_io_buffer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_buffer_operations");
    group.throughput(Throughput::Bytes(1024));

    // Simulate buffer processing
    group.bench_function("buffer_copy_1kb", |b| {
        let data = vec![0u8; 1024];
        b.iter(|| {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(black_box(&data));
            black_box(buffer);
        });
    });

    group.bench_function("buffer_copy_4kb", |b| {
        let data = vec![0u8; 4096];
        group.throughput(Throughput::Bytes(4096));
        b.iter(|| {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(black_box(&data));
            black_box(buffer);
        });
    });

    group.bench_function("buffer_copy_16kb", |b| {
        let data = vec![0u8; 16384];
        group.throughput(Throughput::Bytes(16384));
        b.iter(|| {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(black_box(&data));
            black_box(buffer);
        });
    });

    group.finish();
}

/// Benchmark I/O processing patterns
fn bench_io_processing_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_processing_patterns");

    // Simulate common processing patterns
    group.bench_function("line_by_line_processing", |b| {
        let data = "line1\nline2\nline3\nline4\nline5\n".repeat(100);
        b.iter(|| {
            let lines: Vec<&str> = black_box(&data).lines().collect();
            black_box(lines);
        });
    });

    group.bench_function("chunk_processing_8kb", |b| {
        let data = vec![0u8; 8192];
        b.iter(|| {
            let chunks: Vec<_> = black_box(&data).chunks(1024).collect();
            black_box(chunks);
        });
    });

    group.bench_function("transform_pipeline", |b| {
        let data = b"hello world".repeat(100);
        b.iter(|| {
            let transformed: Vec<u8> =
                black_box(&data).iter().map(|&b| b.to_ascii_uppercase()).collect();
            black_box(transformed);
        });
    });

    group.finish();
}

/// Benchmark async I/O patterns (simulated)
fn bench_async_io_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_io_simulation");

    // Simulate async buffer management
    group.bench_function("backpressure_buffer_check", |b| {
        let max_buffer_size = 64 * 1024;
        let current_size = 32 * 1024;
        b.iter(|| {
            black_box(current_size < max_buffer_size);
        });
    });

    group.bench_function("chunk_size_calculation", |b| {
        let max_buffer = 64 * 1024;
        let current = 48 * 1024;
        b.iter(|| {
            let remaining = max_buffer - current;
            let chunk_size = remaining.min(8 * 1024);
            black_box(chunk_size);
        });
    });

    group.finish();
}

/// Benchmark I/O error handling overhead
fn bench_io_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_error_handling");

    group.bench_function("result_ok_path", |b| {
        b.iter(|| {
            let result: std::io::Result<usize> = Ok(1024);
            black_box(result.is_ok());
        });
    });

    group.bench_function("result_err_path", |b| {
        b.iter(|| {
            let result: std::io::Result<usize> =
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
            black_box(result.is_err());
        });
    });

    group.bench_function("result_unwrap_or", |b| {
        b.iter(|| {
            let result: std::io::Result<usize> = Ok(1024);
            black_box(result.unwrap_or(0));
        });
    });

    group.finish();
}

/// Benchmark I/O type detection and validation
fn bench_io_type_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_type_detection");

    group.bench_function("path_extension_check", |b| {
        let path = "/path/to/file.json";
        b.iter(|| {
            black_box(path.ends_with(".json"));
        });
    });

    group.bench_function("stdin_vs_file_check", |b| {
        let path = "-";
        b.iter(|| {
            black_box(path == "-");
        });
    });

    group.bench_function("format_detection", |b| {
        let content = r#"{"key": "value"}"#;
        b.iter(|| {
            // Simulate JSON detection
            let is_json = content.trim_start().starts_with('{');
            black_box(is_json);
        });
    });

    group.finish();
}

/// Benchmark large file simulation
fn bench_large_file_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_file_simulation");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(15));

    // Simulate processing 1MB file
    group.bench_function("process_1mb_chunks", |b| {
        let data = vec![0u8; 1024 * 1024]; // 1 MB
        group.throughput(Throughput::Bytes(data.len() as u64));

        b.iter(|| {
            let mut processed = 0;
            for chunk in data.chunks(8192) {
                processed += chunk.len();
                black_box(chunk);
            }
            black_box(processed);
        });
    });

    // Simulate processing 10MB file
    group.bench_function("process_10mb_chunks", |b| {
        let data = vec![0u8; 10 * 1024 * 1024]; // 10 MB
        group.throughput(Throughput::Bytes(data.len() as u64));

        b.iter(|| {
            let mut processed = 0;
            for chunk in data.chunks(16384) {
                processed += chunk.len();
                black_box(chunk);
            }
            black_box(processed);
        });
    });

    group.finish();
}

/// Benchmark concurrent I/O patterns
fn bench_concurrent_io_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_io_patterns");

    group.bench_function("multiple_pipeline_creation", |b| {
        b.iter(|| {
            let pipelines: Vec<_> = (0..10).map(|_| IoPipeline::new()).collect();
            black_box(pipelines);
        });
    });

    group.bench_function("pipeline_buffer_size_variety", |b| {
        b.iter(|| {
            let sizes = [4096, 8192, 16384, 32768];
            let pipelines: Vec<_> =
                sizes.iter().map(|&size| IoPipeline::new().with_buffer_size(size)).collect();
            black_box(pipelines);
        });
    });

    group.finish();
}

criterion_group! {
    name = io_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_io_pipeline_construction,
        bench_io_buffer_operations,
        bench_io_processing_patterns,
        bench_async_io_simulation,
        bench_io_error_handling,
        bench_io_type_detection,
        bench_large_file_simulation,
        bench_concurrent_io_patterns
}

criterion_main!(io_benches);
