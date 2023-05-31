use std::fmt::{self, Write};
use std::io::stdin;

use anyhow::Result;
use scraper::Selector;

use crate::{definition, fetch};

/// The Duden.de base url without a trailing slash
const BASE_URL: &str = "https://www.duden.de";

fn url(path: &str) -> String {
    format!("{BASE_URL}/{path}")
}

fn search_url(term: &str) -> String {
    let path = format!("suchen/dudenonline/{term}");
    url(&path)
}

pub fn search(term: &str) -> Result<()> {
    println!("Searching \"{term}\"...");

    let doc = fetch::html(&search_url(term))?;

    let mut results = Vec::new();

    let vignette_selector = Selector::parse(".vignette").unwrap();
    let vignette_word_selector = Selector::parse("strong").unwrap();
    let vignette_source_selector = Selector::parse("a.vignette__label").unwrap();
    let vignette_snippet_selector = Selector::parse(".vignette__snippet").unwrap();

    for vignette in doc.select(&vignette_selector) {
        let word = vignette
            .select(&vignette_word_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap();

        let source = vignette
            .select(&vignette_source_selector)
            .next()
            .and_then(|source| source.value().attr("href"))
            .unwrap();

        let snippet = vignette
            .select(&vignette_snippet_selector)
            .next()
            .and_then(|snippet| snippet.text().next())
            .map(|snippet| snippet.trim());

        let item = Item {
            word,
            source,
            snippet,
        };

        results.push(item);
    }

    if results.is_empty() {
        println!("No results");
        return Ok(());
    }

    for (i, item) in results.iter().enumerate() {
        print!("[{i: >2}] ");
        println!("{item}");
    }

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let selection: usize = input.trim().parse().expect("not a number");

    let Some(result) = results.get(selection) else {
        eprintln!("invalid selection {selection}");
        return Ok(());
    };

    let definition_html = fetch::html(&url(result.source))?;
    let definition = definition::Definition::parse(&definition_html)?;
    println!("{}", definition.title());
    Ok(())
}

struct Item<'s> {
    word: &'s str,
    source: &'s str,
    snippet: Option<&'s str>,
}

impl<'s> fmt::Display for Item<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1m")?;
        // skip soft hyphens
        for c in self.word.chars().filter(|&c| c != '\u{ad}') {
            f.write_char(c)?;
        }
        write!(f, "\x1b[m")?;

        if let Some(snippet) = self.snippet {
            write!(f, " ({})", snippet)?;
        }

        Ok(())
    }
}
