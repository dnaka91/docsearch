use anyhow::Result;
use reqwest::redirect::Policy;

pub async fn search(name: &str) -> Result<(String, String)> {
    let resp = reqwest::Client::builder()
        .redirect(Policy::limited(10))
        .build()?
        .get(format!("https://docs.rs/{}", name))
        .send()
        .await?
        .error_for_status()?;

    let version = resp
        .url()
        .path_segments()
        .unwrap()
        .nth(1)
        .unwrap()
        .to_owned();
    let body = resp.text().await?;

    let index_url = find_url(&body).unwrap();
    println!("url: {}", index_url);
    let index_url = format!("https://docs.rs/{}/{}/{}", name, version, index_url);
    println!("url: {}", index_url);

    let index = reqwest::get(index_url)
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok((version, index))
}

fn find_url(body: &str) -> Option<&str> {
    if let Some(start) = body.find("\"../search-index-") {
        if let Some(end) = body[start..].find(".js\"") {
            return Some(&body[start + 4..start + end + 3]);
        }
    }

    None
}
