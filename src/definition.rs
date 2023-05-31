use std::fmt;

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
        crate::fmt::write_without_shys(&mut title, self.title).unwrap();
        title
    }
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1m")?;
        crate::fmt::write_without_shys(&mut f, self.title)?;
        write!(f, "\x1b[m")?;
        writeln!(f, "\n")?;
        for (i, meaning) in (1..).zip(&self.meanings) {
            writeln!(&mut f, "{i}) {meaning}")?;
        }
        Ok(())
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

#[derive(Debug)]
struct SimpleMeaning<'a> {
    text: scraper::element_ref::Text<'a>,
    usage: Option<&'a str>,
    example: Option<Vec<&'a str>>,
}

impl<'html> SimpleMeaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let text_selector = Selector::parse(".enumeration__text").unwrap();
        let text = html
            .select(&text_selector)
            .next()
            .map(|text| text.text())
            .ok_or(anyhow::anyhow!("simple meaning has no text"))?;

        Ok(Self {
            text,
            usage: None,
            example: None,
        })
    }
}

impl fmt::Display for SimpleMeaning<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::fmt::write_text_trimmed(&mut f, self.text.clone())?;
        Ok(())
    }
}

#[derive(Debug)]
enum Meaning<'a> {
    Simple(SimpleMeaning<'a>),
    Complex(Vec<SimpleMeaning<'a>>),
}

impl fmt::Display for Meaning<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple(meaning) => write!(f, "{meaning}")?,
            Self::Complex(meanings) => {
                for meaning in meanings {
                    writeln!(f, "{meaning}")?;
                }
            }
        }
        Ok(())
    }
}

impl<'html> Meaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let simple_meaning = SimpleMeaning::parse(html)?;
        Ok(Self::Simple(simple_meaning))
    }
}
