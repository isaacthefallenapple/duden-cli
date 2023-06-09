use std::fmt;
use std::io::{prelude::*, stdin};
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

    for vignette in doc.select(selectors::vignette()) {
        let word = vignette
            .select(selectors::vignette_word())
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap();

        let source = vignette
            .select(selectors::vignette_source())
            .next()
            .and_then(|source| source.value().attr("href"))
            .unwrap();

        let snippet = vignette
            .select(selectors::vignette_snippet())
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

        println!("[{i: >2}] {item}");
    }

    let mut input = String::new();
    let selection: usize = loop {
        input.clear();

        stdin().read_line(&mut input)?;

        let Ok(selection) = input.trim().parse() else {
            eprintln!("invalid number: {input}");
            continue;
        };

        if selection >= results.len() {
            eprintln!("invlid selection: {selection}");
            continue;
        }

        break selection;
    };

    let result = rx.iter().find(|(i, _)| i == &selection).map(|res| res.1);

    let result = result.unwrap()?;

    let definition = definition::Definition::parse(result.root_element())?;

    let tempfile_name = {
        let mut tempdir = std::env::temp_dir();
        tempdir.push("duden.tmp");
        tempdir
    };

    let mut temp = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&tempfile_name)?;

    write!(temp, "{definition}")?;
    temp.flush()?;

    temp.rewind()?;

    let (_rows, _) = tty::size();

    println!();

    let less_success = cfg!(unix) && {
        let cmd = std::process::Command::new("/bin/less")
            .args([
                "-RF",
                "-P",
                &definition.title(),
                "--",
                tempfile_name.to_str().unwrap(),
            ])
            .spawn();

        cmd.is_ok_and(|mut cmd| cmd.wait().is_ok())
    };

    if !less_success {
        let mut stdout = std::io::stdout();
        std::io::copy(&mut temp, &mut stdout).unwrap();
        stdout.flush().unwrap();
    }

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
    selector!(vignette = ".vignette");
    selector!(vignette_word = "strong");
    selector!(vignette_source = "a.vignette__label");
    selector!(vignette_snippet = ".vignette__snippet");
}

mod tty {
    #[cfg(unix)]
    pub use unix::size;

    #[cfg(windows)]
    pub use windows::size;

    #[cfg(not(any(unix, windows)))]
    pub fn size() -> (i16, i16) {
        panic!("unsupported platform")
    }

    #[cfg(unix)]
    mod unix {
        use std::process::Stdio;

        pub fn size() -> (i16, i16) {
            // this doesn't strike me as particularly portable(?)
            let stty = std::process::Command::new("/bin/stty")
                .arg("size")
                .stdout(Stdio::piped())
                .spawn()
                .expect("internal error (possibly not a tty)");

            let tty_size = stty.wait_with_output().expect("internal error");
            let tty_size = std::str::from_utf8(&tty_size.stdout).unwrap();
            let mut coords = tty_size
                .split_whitespace()
                .take(2)
                .map(|coord| coord.parse().unwrap());

            (coords.next().unwrap(), coords.next().unwrap())
        }
    }

    #[cfg(windows)]
    mod windows {
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        use winapi::um::wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};

        pub fn size() -> (i16, i16) {
            unsafe {
                let handle = GetStdHandle(STD_OUTPUT_HANDLE);
                let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
                GetConsoleScreenBufferInfo(handle, &mut info as *mut _);
                let coord = info.dwSize;
                (coord.Y as i16, coord.X as i16)
            }
        }
    }
}
