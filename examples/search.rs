//! Simple example that takes a FQN as argument, downloads the index and performs a single search
//! for the FQN.
//!
//! If everything goes well it prints the URL to the requested item.

use std::env;

use docsearch::{Fqn, Result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "docsearch=trace");
    env_logger::init();

    let fqn = parse_args();

    let index = docsearch::search(fqn.crate_name(), None).await?;
    let link = index.find_link(&fqn);

    println!("FQN:  {}", fqn);

    match link {
        Some(link) => println!("Link: {}", link),
        None => println!("Not found :-("),
    }

    Ok(())
}

/// Parse the arguments of this example. Uses panic for the sake of simplicity.
fn parse_args() -> Fqn {
    match env::args().nth(1) {
        Some(fqn) => fqn.parse().unwrap(),
        _ => panic!("Usage: cargo run --example search -- <fqn>"),
    }
}
