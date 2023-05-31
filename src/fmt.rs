use std::fmt;

pub fn write_without_shys(mut w: impl fmt::Write, s: &str) -> fmt::Result {
    s.chars()
        .filter(|&c| c != '\u{ad}')
        .try_for_each(|c| w.write_char(c))
}

pub fn write_text_trimmed(
    mut w: impl fmt::Write,
    mut text: scraper::element_ref::Text<'_>,
) -> fmt::Result {
    let first: Option<&str> = text.next().map(|t| t.trim_start());
    let Some(mut last) = text.next() else {
        if let Some(t) = first {
            write!(w, "{}", t.trim_end())?;
        }
        return Ok(());
    };

    for next in text {
        write!(w, "{last}")?;
        last = next;
    }

    write!(w, "{}", last.trim_end())?;

    Ok(())
}
