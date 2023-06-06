use anyhow::Result;
use scraper::{element_ref::Text, ElementRef};

pub trait FromElement<'a>: Sized {
    fn parse(element: ElementRef<'a>) -> Result<Self>;
}

impl<'a> FromElement<'a> for Text<'a> {
    fn parse(element: ElementRef<'a>) -> Result<Self> {
        Ok(element.text())
    }
}
