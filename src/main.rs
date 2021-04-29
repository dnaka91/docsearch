use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let result = if let Some(name) = std::env::args().nth(1) {
        docsearch::search(&name, None).await?
    } else {
        docsearch::get_std().await?
    };

    println!("{:#?}", result);
    Ok(())
}
