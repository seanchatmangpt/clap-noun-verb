//! Async I/O Example
//!
//! Demonstrates advanced async I/O patterns with Tokio:
//! - Backpressure-aware reading
//! - Bidirectional streams
//! - Length-delimited framing
//! - High-performance processing
//!
//! # Running
//!
//! ```bash
//! # Echo server demo
//! cargo run --example async_io_example -- echo-server
//!
//! # In another terminal:
//! echo "Hello, Async World!" | nc localhost 8888
//! ```

use clap::{Parser, Subcommand};
use clap_noun_verb::io::async_io::{
    AsyncInputExt, AsyncOutputExt, BackpressureConfig, LengthDelimitedFrameBuilder,
    LinesFrameBuilder,
};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser)]
#[command(name = "async-io-demo")]
#[command(about = "Async I/O patterns with clap-noun-verb")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Echo server with backpressure handling
    EchoServer {
        /// Listen address
        #[arg(short, long, default_value = "127.0.0.1:8888")]
        addr: String,
    },

    /// Process stdin with backpressure
    Transform {
        /// Transformation type (uppercase, lowercase, reverse)
        #[arg(short, long, default_value = "uppercase")]
        kind: String,
    },

    /// Test framed I/O with length-delimited messages
    FramedEcho {
        /// Listen address
        #[arg(short, long, default_value = "127.0.0.1:8889")]
        addr: String,
    },

    /// Benchmark async throughput
    Benchmark {
        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Buffer size in KB
        #[arg(short, long, default_value = "64")]
        buffer_kb: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::EchoServer { addr } => {
            echo_server(&addr).await?;
        }

        Commands::Transform { kind } => {
            transform_stdin(&kind).await?;
        }

        Commands::FramedEcho { addr } => {
            framed_echo_server(&addr).await?;
        }

        Commands::Benchmark { iterations, buffer_kb } => {
            benchmark_throughput(iterations, buffer_kb * 1024).await?;
        }
    }

    Ok(())
}

/// Simple echo server demonstrating backpressure handling
async fn echo_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Echo server listening on {}", addr);
    eprintln!("To test: echo 'hello' | nc {}", addr);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        eprintln!("New connection from: {}", peer_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_echo_client(socket).await {
                eprintln!("Error handling client {}: {}", peer_addr, e);
            }
        });
    }
}

/// Handle single echo client with backpressure
async fn handle_echo_client(mut socket: TcpStream) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let config = BackpressureConfig::new().with_max_buffer(32 * 1024).with_chunk_size(4 * 1024);

    while reader.read_line(&mut line).await? > 0 {
        // Echo with backpressure awareness
        writer.write_with_backpressure(line.as_bytes(), &config).await?;
        line.clear();
    }

    Ok(())
}

/// Transform stdin with backpressure
async fn transform_stdin(kind: &str) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    let config = BackpressureConfig::new().with_max_buffer(128 * 1024).with_chunk_size(8 * 1024);

    while reader.read_line(&mut line).await? > 0 {
        let transformed = match kind {
            "uppercase" => line.to_uppercase(),
            "lowercase" => line.to_lowercase(),
            "reverse" => line.chars().rev().collect(),
            _ => line.clone(),
        };

        stdout.write_with_backpressure(transformed.as_bytes(), &config).await?;
        line.clear();
    }

    stdout.flush().await?;
    Ok(())
}

/// Framed echo server with length-delimited messages
async fn framed_echo_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Framed echo server listening on {}", addr);

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        eprintln!("Framed connection from: {}", peer_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_framed_client(socket).await {
                eprintln!("Error handling framed client {}: {}", peer_addr, e);
            }
        });
    }
}

/// Handle framed client
async fn handle_framed_client(mut socket: TcpStream) -> anyhow::Result<()> {
    let builder = LengthDelimitedFrameBuilder::new().with_max_size(1_000_000);
    let mut buffer = vec![0u8; 8192];
    let mut accumulated = Vec::new();

    loop {
        match socket.read(&mut buffer).await? {
            0 => break,
            n => {
                accumulated.extend_from_slice(&buffer[..n]);

                while let Some((len, _)) = builder.parse(&accumulated)? {
                    let frame_size = 4 + len as usize;
                    if accumulated.len() >= frame_size {
                        let frame = accumulated.split_off(frame_size);
                        socket.write_all(&accumulated).await?;
                        socket.flush().await?;
                        accumulated = frame;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Benchmark async throughput
async fn benchmark_throughput(iterations: usize, buffer_size: usize) -> anyhow::Result<()> {
    let data = vec![0u8; buffer_size];
    let start = std::time::Instant::now();

    let mut tasks = vec![];

    for _ in 0..iterations {
        let data = data.clone();
        let task = tokio::spawn(async move {
            let buffer = data;
            buffer.len()
        });
        tasks.push(task);
    }

    let mut total = 0usize;
    for task in tasks {
        total += task.await?;
    }

    let elapsed = start.elapsed();
    let throughput_mbs = (total as f64) / elapsed.as_secs_f64() / 1_000_000.0;

    println!("Benchmark Results:");
    println!("  Iterations: {}", iterations);
    println!("  Buffer size: {} KB", buffer_size / 1024);
    println!("  Total data: {} MB", total / 1_000_000);
    println!("  Elapsed: {:?}", elapsed);
    println!("  Throughput: {:.2} MB/s", throughput_mbs);

    Ok(())
}
