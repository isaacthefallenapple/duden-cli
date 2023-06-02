use anyhow::Result;
use reqwest::blocking as reqwest;
use scraper::Html;

pub fn html(client: &reqwest::Client, from: &str) -> Result<Html> {
    let res = client.get(from).send()?;
    Ok(Html::parse_document(&res.text()?))
}
