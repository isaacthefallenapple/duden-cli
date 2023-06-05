use std::fmt;

use crate::selector::{selector, selectors};
use anyhow::Result;
use scraper::{element_ref::Text, ElementRef, Html};

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
        let title = html
            .select(selectors::title_selector())
            .next()
            .and_then(|title| title.text().next())
            .expect("definition doesn't have title");

        let mut meanings = Vec::new();

        for meaning in html.select(selectors::meanings_selector()) {
            meanings.push(Meaning::parse(meaning)?);
        }

        if meanings.is_empty() {
            let singleton_meaning = html
                .select(selectors::singleton_meaning())
                .next()
                .expect("definition has no meaning at all");

            meanings.push(Meaning::Simple(SimpleMeaning::new(
                singleton_meaning.text(),
            )));
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
    fn new(text: Text<'html>) -> Self {
        Self {
            text,
            example: None,
            usage: None,
        }
    }

    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let text = html
            .select(selectors::text_selector())
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
        let width = f.width().unwrap_or_default();

        for _ in 0..width {
            write!(&mut f, "\t")?;
        }
        if let Some(nesting) = f.precision() {
            write!(&mut f, "{}) ", (b'a' + nesting as u8) as char)?;
        }
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
        let indent = f.width().unwrap_or_default();
        match self {
            Self::Simple(meaning) => write!(f, "{meaning:indent$}", indent = indent)?,
            Self::Complex(meanings) => {
                for (i, meaning) in (0..).zip(meanings) {
                    writeln!(f, "{meaning:indent$.index$}", indent = 1, index = i)?;
                }
            }
        }
        Ok(())
    }
}

impl<'html> Meaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let mut sub_items = Vec::new();

        for sub_item in html.select(selectors::sub_item_selector()) {
            sub_items.push(SimpleMeaning::parse(sub_item)?);
        }

        if sub_items.is_empty() {
            Ok(Self::Simple(SimpleMeaning::parse(html)?))
        } else {
            Ok(Self::Complex(sub_items))
        }
    }
}

struct Tuple<'a> {
    key: Text<'a>,
    val: ElementRef<'a>,
}

impl fmt::Display for Tuple<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::fmt::write_text_trimmed(&mut f, self.key.clone())?;
        writeln!(&mut f)?;
        crate::fmt::write_text_trimmed(&mut f, self.val.text())?;
        Ok(())
    }
}

fn parse_tuples(root: ElementRef<'_>) -> Result<Tuple<'_>> {
    let key = root
        .select(selectors::tuple_key())
        .next()
        .ok_or(anyhow::anyhow!("tuple doesn't have a key"))?
        .text();

    let val = root
        .select(selectors::tuple_val())
        .next()
        .ok_or(anyhow::anyhow!("tuple doesn't have a val"))?;

    Ok(Tuple { key, val })
}

selectors! {
    selector!(text_selector = ".enumeration__text");
    selector!(title_selector = "h1 > span");
    selector!(meanings_selector = "#bedeutungen .enumeration__item");
    selector!(sub_item_selector = ".enumeration__sub-item");
    selector!(tuple_key = "dt.tuple__key");
    selector!(tuple_val = "dd.tuple__val");
    selector!(singleton_meaning = "#bedeutung p");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tuple() -> Result<()> {
        let html = r#"
            <dl class="tuple">
                <dt class="tuple__key">Gebrauch</dt>
                <dd class="tuple__val">Chemie</dd>
            </dl>
        "#;

        let fragment = Html::parse_fragment(html);
        let root = fragment.root_element();

        let tuple = parse_tuples(root)?;
        eprintln!("{}", tuple);
        panic!();
        Ok(())
    }
}
