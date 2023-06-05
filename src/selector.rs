macro_rules! selectors {
    ($($tokens:tt)*) => {
        mod selectors {
            use super::*;

            use ::std::sync::OnceLock;

            use ::paste::paste;
            use ::scraper::Selector;

            $($tokens)*
        }
    };
}

macro_rules! selector {
    ($name:ident = $value:literal) => {
        paste! {
            static [<$name:upper>]: OnceLock<Selector> = OnceLock::new();

            #[doc = concat!("Select ", $value)]
            pub fn [<$name:lower>]() -> &'static Selector {
                [<$name:upper>].get_or_init(|| Selector::parse($value).unwrap())
            }
        }
    };
}

pub(crate) use selector;
pub(crate) use selectors;
