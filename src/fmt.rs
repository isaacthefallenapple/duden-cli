use std::fmt;

pub fn write_no_shys(mut w: impl fmt::Write, s: &str) -> fmt::Result {
    s.chars()
        .filter(|&c| c != '\u{ad}')
        .try_for_each(|c| w.write_char(c))
}
