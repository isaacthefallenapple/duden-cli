use std::fmt;

pub fn write_no_shys(mut w: impl fmt::Write, s: &str) -> fmt::Result {
    s.chars()
        .filter(|&c| c != '\u{ad}')
        .try_for_each(|c| w.write_char(c))
}

pub fn write_text_trimmed(
    mut w: impl fmt::Write,
    mut text: scraper::element_ref::Text<'_>,
) -> fmt::Result {
    let first: Option<&str> = text.next().map(|t| t.trim_start());
    let mut last: Option<&str> = text.next();

    if last.is_none() {
        if let Some(t) = first {
            write!(w, "{}", t.trim_end())?;
        }
        return Ok(());
    }

    while let next @ Some(_) = text.next() {
        let t = last.unwrap();
        write!(w, "{t}")?;
        last = next;
    }

    let t = last.unwrap();
    write!(w, "{t}")?;

    Ok(())
}
