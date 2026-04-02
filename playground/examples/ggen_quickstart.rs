//! ggen v26.4.2 Quickstart Example
//!
//! Demonstrates basic ggen workflow:
//! 1. Enable a capability
//! 2. Add required packs
//! 3. Run sync
//! 4. Verify receipt

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ggen v26.4.2 Quickstart");
    println!();

    println!("1. Enable MCP capability:");
    println!("   $ ggen capability enable --surface mcp --projection rust");
    println!();

    println!("2. Add required packs:");
    println!("   $ ggen pack add mcp-server");
    println!();

    println!("3. Run sync (sacred command):");
    println!("   $ ggen sync");
    println!();

    println!("4. Verify receipt:");
    println!("   $ ggen receipt verify --file receipts/<id>.json");
    println!();

    Ok(())
}
