use std::fmt::{self, Write};
use std::io::stdin;

use anyhow::Result;
use scraper::Selector;

use crate::selector::{selector, selectors};
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

    for vignette in doc.select(selectors::vignette_selector()) {
        let word = vignette
            .select(selectors::vignette_word_selector())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap();

        let source = vignette
            .select(selectors::vignette_source_selector())
            .next()
            .and_then(|source| source.value().attr("href"))
            .unwrap();

        let snippet = vignette
            .select(selectors::vignette_snippet_selector())
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
    print!("\n{}", definition);

    Ok(())
}

struct Item<'s> {
    word: &'s str,
    source: &'s str,
    snippet: Option<&'s str>,
}

impl<'s> fmt::Display for Item<'s> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1m")?;
        // skip soft hyphens
        crate::fmt::write_without_shys(&mut f, self.word)?;
        write!(f, "\x1b[m")?;

        if let Some(snippet) = self.snippet {
            write!(f, " ({})", snippet)?;
        }

        Ok(())
    }
}

selectors! {
    selector!(vignette_selector = ".vignette");
    selector!(vignette_word_selector = "strong");
    selector!(vignette_source_selector = "a.vignette__label");
    selector!(vignette_snippet_selector = ".vignette__snippet");
}
