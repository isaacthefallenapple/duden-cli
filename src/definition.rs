use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct Definition<'a> {
    title: &'a str,
    // TODO: implement this
    // properties: Properties<'a>,
    // spelling: Spelling<'a>,
    meanings: Vec<Meaning<'a>>,
}

impl Definition<'_> {
    pub fn title(&self) -> String {
        let mut title = String::with_capacity(self.title.len());
        crate::fmt::write_no_shys(&mut title, self.title).unwrap();
        title
    }
}

impl<'html> Definition<'html> {
    pub fn parse(html: &'html Html) -> Result<Self> {
        let title_selector = Selector::parse("h1 > span").unwrap();
        let meanings_selector = Selector::parse("#bedeutungen .enumeration__item").unwrap();

        let title = html
            .select(&title_selector)
            .next()
            .and_then(|title| title.text().next())
            .expect("definition doesn't have title");

        let mut meanings = Vec::new();

        for meaning in html.select(&meanings_selector) {
            meanings.push(Meaning::parse(meaning)?);
        }

        Ok(Self { title, meanings })
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

#[derive(Default, Debug)]
struct SimpleMeaning<'a> {
    text: String,
    usage: Option<&'a str>,
    example: Option<Vec<&'a str>>,
}

impl<'html> SimpleMeaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let text_selector = Selector::parse(".enumeration__text").unwrap();
        let text = html
            .select(&text_selector)
            .next()
            .map(|text| text.text().collect())
            .ok_or(anyhow::anyhow!("simple meaning has no text"))?;

        Ok(Self {
            text,
            ..Self::default()
        })
    }
}

#[derive(Debug)]
enum Meaning<'a> {
    Simple(SimpleMeaning<'a>),
    Complex(Vec<SimpleMeaning<'a>>),
}

impl<'html> Meaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let simple_meaning = SimpleMeaning::parse(html)?;
        Ok(Self::Simple(simple_meaning))
    }
}
