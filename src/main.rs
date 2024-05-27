use clap::Parser;
use minigrep::Cli;
use regex::Regex;
use std::{
    fs,
    io::{self, BufRead},
};

fn main() {
    let args = Cli::parse();

    let content = match args.path {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }),
        None => io::stdin()
            .lock()
            .lines()
            .fold(String::new(), |acc, line| {
                acc + &line.unwrap_or_else(|e| panic!("{e} occured.")) + "\n"
            }),
    };

    let re = match args.ignore_case {
        true => Regex::new(&format!("(?i){}", &args.pattern)),
        false => Regex::new(&args.pattern),
    }
    .unwrap_or_else(|_| panic!("oops something went wrong. crazy ikr."));

    let res = minigrep::search(&re, &content);
    let output = minigrep::stdout_print(res, &re, false);

    println!("{output}");
}
