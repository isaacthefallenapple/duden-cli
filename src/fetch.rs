use anyhow::Result;
use reqwest::blocking as reqwest;
use scraper::Html;

pub fn html(from: &str) -> Result<Html> {
    let res = reqwest::get(from)?;
    Ok(Html::parse_document(&res.text()?))
}
