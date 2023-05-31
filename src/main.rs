use std::env;

use anyhow::Result;

mod fetch;
mod search;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let Some(subcommand) = args.next() else {
        anyhow::bail!("no command given");
    };

    match &*subcommand {
        "search" => {
            let Some(argument) = args.next() else {
                anyhow::bail!("missing argument for `search` command");
            };
            search::search(&argument)?;
        }
        _ => {
            anyhow::bail!("unknown command: `{}`", subcommand);
        }
    }

    Ok(())
}
