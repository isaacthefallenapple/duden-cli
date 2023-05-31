use std::fmt::{self, Write};
use std::io::stdin;

use anyhow::Result;
use scraper::Selector;

use crate::fetch;

pub fn search(term: &str) -> Result<()> {
    println!("Searching \"{term}\"...");

    let doc = fetch::html(&format!("https://www.duden.de/suchen/dudenonline/{term}"))?;

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

    println!("Find more at {}", result.source);
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
