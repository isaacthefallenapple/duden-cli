use std::env;

mod search;

fn main() {
    let mut args = env::args().skip(1);

    let Some(subcommand) = args.next() else {
        return;
    };

    match &*subcommand {
        "search" => {
            let Some(argument) = args.next() else {
                return;
            };
            search::search(&argument);
        }
        _ => {
            eprintln!("Unknown command: \"{subcommand}\"");
            return;
        }
    }

    println!("{}", subcommand);
}
