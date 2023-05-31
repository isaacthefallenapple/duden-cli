macro_rules! selectors {
    ($($tokens:tt)*) => {
        mod selectors {
            use super::*;

            use std::{mem::MaybeUninit, sync::Once};

            use paste::paste;
            use scraper::Selector;

            $($tokens)*
        }
    };
}

macro_rules! selector {
    ($name:ident = $value:literal) => {
        paste! {
            static mut [<$name:upper>]: MaybeUninit<Selector> = MaybeUninit::uninit();
            static [<$name:upper _INIT>]: Once = Once::new();

            #[doc = concat!("Select ", $value)]
            pub fn [<$name:lower>]() -> &'static Selector {
                unsafe {
                    [<$name:upper _INIT>].call_once(|| {
                        [<$name:upper>].write(Selector::parse($value).unwrap());
                    });
                    [<$name:upper>].assume_init_ref()
                }
            }
        }
    };
}

pub(crate) use selectors;
pub(crate) use selector;
