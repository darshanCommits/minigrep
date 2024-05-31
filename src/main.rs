use clap::Parser;
use minigrep::{Cli, Colorize, Grepped};
use regex::Regex;

fn main() {
    let args = Cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("{}: {}", "error".red(), e);
        std::process::exit(1);
    }
}

fn run(args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let searchable = &args.to_searchable()?;
    let searchable_iter = searchable.search();
    let output = stdout_str(searchable_iter, &searchable.re, false);

    println!("{output}");

    Ok(())
}

pub fn stdout_str<'a>(
    arr: impl Iterator<Item = Grepped<'a>>,
    re: &'a Regex,
    test: bool,
) -> String {
    arr.map(|grepped| match test {
        false => grepped.to_colored(re),
        true => grepped.to_non_colored(),
    })
    .fold(String::new(), |mut acc, s| {
        acc.push_str(&s);
        acc
    })
}
