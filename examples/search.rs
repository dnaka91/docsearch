//! Simple example that takes an item path as argument, downloads the index and performs a single
//! search on the given argument.
//!
//! If everything goes well it prints the URL to the requested item.

use std::env;

use anyhow::Result;
use docsearch::{Index, SimplePath, Version};
use reqwest::redirect::Policy;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "docsearch=trace");
    env_logger::init();

    let path = parse_args();

    let index = search(path.crate_name(), Version::Latest).await?;
    let link = index.find_link(&path);

    println!("Path: {path}");

    match link {
        Some(link) => println!("Link: {link}"),
        None => println!("Not found :-("),
    }

    Ok(())
}

async fn search(name: &str, version: Version) -> Result<Index> {
    let state = docsearch::start_search(name, version);
    let content = reqwest::Client::builder()
        .redirect(Policy::limited(10))
        .build()?
        .get(state.url())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let state = state.find_index(&content)?;
    let content = reqwest::Client::builder()
        .build()?
        .get(state.url())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    state.transform_index(&content).map_err(Into::into)
}

/// Parse the arguments of this example. Uses panic for the sake of simplicity.
fn parse_args() -> SimplePath {
    match env::args().nth(1) {
        Some(path) => path.parse().unwrap(),
        _ => panic!("Usage: cargo run --example search -- <path>"),
    }
}
