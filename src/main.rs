use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Some(name) = std::env::args().nth(1) {
        let result = docsearch::search(&name).await?;
        println!("{:#?}", result);
    } else {
        eprintln!("Usage: docsearch <crate_name>")
    }

    Ok(())
}
