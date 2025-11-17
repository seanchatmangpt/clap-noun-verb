use clap_noun_verb::{noun, verb, Result};

#[noun("services", "Manage services")]
fn services() {}

#[verb("list")]
fn list_services(names: Vec<String>) -> Result<()> {
    for name in names {
        println!("Service: {}", name);
    }
    Ok(())
}

fn main() {
    println!("Vec<String> parsing test compiled!");
}
