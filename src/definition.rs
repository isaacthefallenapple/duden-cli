use std::fmt;

use crate::parse::FromElement;
use crate::selector::{selector, selectors};
use anyhow::Result;
use scraper::{element_ref::Text, ElementRef};

#[derive(Debug)]
pub struct Definition<'a> {
    title: Text<'a>,
    // TODO: implement this
    // properties: _Properties<'a>,
    // spelling: _Spelling<'a>,
    meanings: Vec<Meaning<'a>>,
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1m")?;
        crate::fmt::write_text_trimmed(&mut f, true, self.title.clone())?;
        writeln!(f, "\x1b[m\n")?;

        for (i, meaning) in (1..).zip(&self.meanings) {
            writeln!(&mut f, "{i}) {meaning}")?;
        }
        Ok(())
    }
}

impl<'html> FromElement<'html> for Definition<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let title = html
            .select(selectors::title_selector())
            .next()
            .map(|title| title.text())
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

struct _Properties<'a> {
    part_of_speech: &'a str,
    frequency: &'a str,
    pronunciation: &'a str,
}

struct _Spelling<'a> {
    variants: Vec<&'a str>,
    related: Vec<&'a str>,
}

#[derive(Debug)]
struct SimpleMeaning<'a> {
    text: scraper::element_ref::Text<'a>,
    _usage: Option<&'a str>,
    _example: Option<Vec<Text<'a>>>,
}

impl<'html> SimpleMeaning<'html> {
    fn new(text: Text<'html>) -> Self {
        Self {
            text,
            _example: None,
            _usage: None,
        }
    }
}

impl<'html> FromElement<'html> for SimpleMeaning<'html> {
    fn parse(html: ElementRef<'html>) -> Result<Self> {
        let text = html
            .select(selectors::text_selector())
            .next()
            .map(|text| text.text())
            .ok_or(anyhow::anyhow!("simple meaning has no text"))?;

        let note = html.select(selectors::note()).next();

        let mut examples = None;

        if let Some(note) = note {
            let mut example_list = Vec::new();
            for li in note.select(selectors::list_item()) {
                example_list.push(li.text());
            }
            examples = Some(example_list);
        }

        Ok(Self {
            text,
            _usage: None,
            _example: examples,
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
        crate::fmt::write_text_trimmed(&mut f, true, self.text.clone())?;

        if let Some(examples) = &self._example {
            write!(
                &mut f,
                "\n\t\x1b[1mBeispiel{plural}\x1b[m",
                plural = if examples.len() > 1 { "e" } else { "" }
            )?;
            for ex in examples {
                write!(&mut f, "\n\t - ")?;
                crate::fmt::write_text_trimmed(&mut f, true, ex.clone())?;
            }
        }
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

impl<'html> FromElement<'html> for Meaning<'html> {
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

struct Tuple<'a, Val = ElementRef<'a>> {
    key: Text<'a>,
    val: Val,
}

impl<'a> Tuple<'a> {
    fn _parse_val<Q: FromElement<'a>>(self) -> Result<Tuple<'a, Q>> {
        let Tuple { key, val } = self;
        Ok(Tuple {
            key,
            val: Q::parse(val)?,
        })
    }
}

impl<'html> FromElement<'html> for Tuple<'html> {
    fn parse(element: ElementRef<'html>) -> Result<Self> {
        let key = element
            .select(selectors::tuple_key())
            .next()
            .ok_or(anyhow::anyhow!("tuple doesn't have a key"))?
            .text();

        let val = element
            .select(selectors::tuple_val())
            .next()
            .ok_or(anyhow::anyhow!("tuple doesn't have a val"))?;

        Ok(Tuple { key, val })
    }
}

impl fmt::Display for Tuple<'_> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::fmt::write_text_trimmed(&mut f, true, self.key.clone())?;
        writeln!(&mut f)?;
        crate::fmt::write_text_trimmed(&mut f, true, self.val.text())?;
        Ok(())
    }
}

selectors! {
    selector!(text_selector = ".enumeration__text");
    selector!(title_selector = "h1");
    selector!(meanings_selector = "#bedeutungen .enumeration__item");
    selector!(sub_item_selector = ".enumeration__sub-item");
    selector!(tuple_key = "dt.tuple__key");
    selector!(tuple_val = "dd.tuple__val");
    selector!(singleton_meaning = "#bedeutung p");
    selector!(note = ".note");
    selector!(list_item = "li");
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

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

        let tuple = Tuple::parse(root)?;
        eprintln!("{}", tuple);
        panic!();
    }
}
