use std::fmt;

pub fn write_without_shys(mut w: impl fmt::Write, s: &str) -> fmt::Result {
    s.chars()
        .filter(|&c| c != '\u{ad}')
        .try_for_each(|c| w.write_char(c))
}

pub fn write_text_trimmed(
    mut w: impl fmt::Write,
    skip_shys: bool,
    mut text: scraper::element_ref::Text<'_>,
) -> fmt::Result {
    let mut write = |s| {
        if skip_shys {
            write_without_shys(&mut w, s)
        } else {
            write!(&mut w, "{s}")
        }
    };

    let Some(first)= text.next().map(|t| t.trim_start()) else {
        return Ok(());
    };

    let Some(mut last) = text.next() else {
        write(first.trim_end())?;
        return Ok(());
    };

    write(first)?;

    for next in text {
        write(last)?;
        last = next;
    }

    write(last.trim_end())?;

    Ok(())
}
