//! Real, runnable server for the autofde-lab-fabric example: stands up
//! clap-noun-verb-deploy's real HttpServer (src/http.rs) over a real TCP
//! listener, serving the same wrap() the integration test already proves.
//! Run: cargo run --example serve --manifest-path
//!   ~/clap-noun-verb/clap-noun-verb-any/Cargo.toml -- 127.0.0.1:8080

use clap_noun_verb_any::wrap;
use clap_noun_verb_deploy::http::HttpServer;
use std::net::TcpListener;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/autofde_lab_planners");
    let executable = example.join("autofde-lab-fabric.sh");
    let manifest = example.join("cnv-any.json");
    let wrapped = wrap(executable.into_os_string(), &manifest)?;
    let (deploy, executor) = wrapped.into_parts();
    let schema = deploy.schema().clone();
    let server = HttpServer::new(schema, executor);

    let bind = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_owned());
    let listener = TcpListener::bind(&bind)?;
    eprintln!("cnv-deploy serving autofde-lab-fabric on http://{bind}");
    server.serve(listener)?;
    Ok(())
}
