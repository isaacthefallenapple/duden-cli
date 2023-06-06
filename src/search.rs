use std::fmt;
use std::io::{prelude::*, stdin, BufReader, SeekFrom};
use std::sync::mpsc;
use std::thread;

use crate::parse::FromElement;
use crate::selector::{selector, selectors};
use crate::{definition, fetch};
use anyhow::Result;
use reqwest::blocking as reqwest;

/// The Duden.de base url without a trailing slash
const BASE_URL: &str = "https://www.duden.de";

fn url(path: &str) -> String {
    format!("{BASE_URL}/{path}", path = path.trim_start_matches('/'))
}

fn search_url(term: &str) -> String {
    let path = format!("suchen/dudenonline/{term}");
    url(&path)
}

pub fn search(client: &reqwest::Client, term: &str) -> Result<()> {
    println!("Searching \"{term}\"...");

    let doc = fetch::html(client, &search_url(term))?;

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

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(results.len());

    for (i, item) in results.iter().enumerate() {
        let tx = tx.clone();
        let source = url(item.source);
        let client = client.clone();

        let handle = thread::spawn(move || {
            prefetch(client, i, source, tx);
        });

        handles.push(handle);

        print!("[{i: >2}] ");
        println!("{item}");
    }

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let selection: usize = input.trim().parse().expect("not a number");

    if selection >= results.len() {
        anyhow::bail!("invlid selection: {selection}");
    }

    let result = rx
        .try_iter()
        .find(|(i, _)| i == &selection)
        .or_else(|| rx.iter().find(|(i, _)| i == &selection))
        .map(|res| res.1);

    let result = result.unwrap()?;

    let definition = definition::Definition::parse(result.root_element())?;

    let tempfile_name = "/tmp/duden.tmp";
    let mut temp = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(tempfile_name)?;

    write!(temp, "{definition}")?;
    temp.flush()?;

    let mut cmd = std::process::Command::new("/bin/less")
        .arg("-R")
        .arg(tempfile_name)
        .spawn()?;

    drop(cmd.wait());

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

fn prefetch(
    client: reqwest::Client,
    id: usize,
    source: String,
    sender: mpsc::Sender<(usize, Result<scraper::Html>)>,
) {
    let fetched = fetch::html(&client, &source);
    // TODO: figure out what to do here on error (can't propagate it up)
    let _ = sender.send((id, fetched));
}

selectors! {
    selector!(vignette_selector = ".vignette");
    selector!(vignette_word_selector = "strong");
    selector!(vignette_source_selector = "a.vignette__label");
    selector!(vignette_snippet_selector = ".vignette__snippet");
}
