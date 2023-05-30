use reqwest::blocking as reqwest;
use scraper::{Html, Selector};

struct Item<'s> {
    word: &'s str,
    source: &'s str,
}

pub fn search(term: &str) {
    println!("Searching \"{term}\"...");
    let res = reqwest::get(&format!("https://www.duden.de/suchen/dudenonline/{term}"));
    let res = res.unwrap();
    let text = res.text().unwrap();

    let doc = Html::parse_document(&text);
    // println!("{doc:#?}");
    let vignette_selector = Selector::parse(".vignette").unwrap();
    let vignette_word_selector = Selector::parse("strong").unwrap();
    let vignette_source_selector = Selector::parse("a.vignette__label").unwrap();
    for vignette in doc.select(&vignette_selector) {
        // eprintln!("{vignette:#?}");
        let word = vignette
            .select(&vignette_word_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap();
        let source = vignette.select(&vignette_source_selector).next().unwrap();
        let source = source.value().attr("href").unwrap();

        println!("{word} @ {source}");
    }
}
