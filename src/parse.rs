use anyhow::Result;
use scraper::ElementRef;

pub trait FromElement<'a>: Sized {
    fn parse(element: ElementRef<'a>) -> Result<Self>;
}
