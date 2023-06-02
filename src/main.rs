use std::env;

use anyhow::Result;
use reqwest::blocking as reqwest;

mod definition;
mod fetch;
mod fmt;
mod search;
mod selector;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let Some(subcommand) = args.next() else {
        anyhow::bail!("no command given");
    };

    let client = reqwest::Client::new();

    match &*subcommand {
        "search" => {
            let Some(argument) = args.next() else {
                anyhow::bail!("missing argument for `search` command");
            };
            search::search(&client, &argument)?;
        }
        _ => {
            anyhow::bail!("unknown command: `{}`", subcommand);
        }
    }

    Ok(())
}
