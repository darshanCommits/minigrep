use clap::Parser;
use core::panic;
use minigrep::Cli;
use regex::Regex;

fn main() {
    let args = Cli::parse();

    let content = std::fs::read_to_string(&args.path).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    });

    let re = match args.ignore_case {
        true => Regex::new(&format!("(?i){}", &args.pattern)),
        false => Regex::new(&args.pattern),
    }
    .unwrap_or_else(|_| panic!("oops something went wrong. crazy ikr."));

    let res = minigrep::search(&re, &content);
    let output = minigrep::stdout_print(res, &re, false);

    println!("{output}");
}
