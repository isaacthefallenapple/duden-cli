use anyhow::Result;
use scraper::{Html, Selector};

pub struct Definition<'a> {
    title: &'a str,
    // TODO: implement this
    // properties: Properties<'a>,
    // spelling: Spelling<'a>,
    meanings: Vec<Meaning<'a>>,
}

impl Definition<'_> {
    pub fn title(&self) -> &str {
        self.title
    }
}

impl<'html> Definition<'html> {
    pub fn parse(html: &'html Html) -> Result<Self> {
        let title_selector = Selector::parse("h1 > span").unwrap();
        let _meanings_selector = Selector::parse("#bedeutungen").unwrap();

        let title = html
            .select(&title_selector)
            .next()
            .and_then(|title| title.text().next())
            .expect("definition doesn't have title");

        Ok(Self {
            title,
            meanings: Vec::new(),
        })
    }
}

struct Properties<'a> {
    part_of_speech: &'a str,
    frequency: &'a str,
    pronunciation: &'a str,
}

struct Spelling<'a> {
    variants: Vec<&'a str>,
    related: Vec<&'a str>,
}

struct SimpleMeaning<'a> {
    text: &'a str,
    usage: Option<&'a str>,
    example: Option<Vec<&'a str>>,
}

enum Meaning<'a> {
    Simple(SimpleMeaning<'a>),
    Complex(Vec<SimpleMeaning<'a>>),
}
