//! Simple example that takes an item path as argument, downloads the index and performs a single
//! search on the given argument.
//!
//! If everything goes well it prints the URL to the requested item.

use std::env;

use docsearch::{Result, SimplePath};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "docsearch=trace");
    env_logger::init();

    let path = parse_args();

    let index = docsearch::search(path.crate_name(), None).await?;
    let link = index.find_link(&path);

    println!("Path: {}", path);

    match link {
        Some(link) => println!("Link: {}", link),
        None => println!("Not found :-("),
    }

    Ok(())
}

/// Parse the arguments of this example. Uses panic for the sake of simplicity.
fn parse_args() -> SimplePath {
    match env::args().nth(1) {
        Some(path) => path.parse().unwrap(),
        _ => panic!("Usage: cargo run --example search -- <path>"),
    }
}
