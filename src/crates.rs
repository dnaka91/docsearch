use anyhow::Result;
use reqwest::redirect::Policy;

const DOCSRS_URL: &str = "https://docs.rs";

pub async fn search(name: &str, version: Option<&str>) -> Result<(String, String)> {
    let page_url = version
        .map(|v| format!("{}/{}/{}", DOCSRS_URL, name, v))
        .unwrap_or_else(|| format!("{}/{}", DOCSRS_URL, name));

    let resp = reqwest::Client::builder()
        .redirect(Policy::limited(10))
        .build()?
        .get(page_url)
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
    let index_url = format!("{}/{}/{}/{}", DOCSRS_URL, name, version, index_url);
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
