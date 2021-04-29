use std::env;

use docsearch::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "docsearch=trace");
    env_logger::init();

    let result = if let Some(name) = env::args().nth(1) {
        docsearch::search(&name, None).await?
    } else {
        docsearch::get_std().await?
    };

    println!("{:#?}", result);
    Ok(())
}
