use std::{
    fs,
    io::{self, BufRead},
};

use clap::Parser;
use regex::Regex;

/// Search for a pattern and stdout
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// The pattern to look for
    pub pattern: String,

    /// File path
    pub path: Option<std::path::PathBuf>,

    /// Case insensitivity?
    #[arg(short, long, env = "IGNORE_CASE", default_value_t = false)]
    ignore_case: bool,

    /// Match whole words?
    #[arg(short, long, default_value_t = false)]
    whole_word: bool,

    /// Match whole lines?
    #[arg(short = 'x', long, default_value_t = false)]
    whole_line: bool,
}

impl Cli {
    fn pattern_modifier<F>(&self, predicate: bool, modifier: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        let pattern = if predicate {
            modifier(&self.pattern)
        } else {
            self.pattern.clone()
        };

        Self {
            pattern,
            path: self.path.clone(),
            ..*self
        }
    }

    pub fn to_searchable(
        &self,
    ) -> Result<Searchable, Box<dyn std::error::Error>> {
        let re = self.build_pattern()?;

        let buffer = match &self.path {
            Some(path) => fs::read_to_string(path)?,
            None => io::stdin().lock().lines().try_fold(
                String::new(),
                |mut acc, line| {
                    line.map(|s| {
                        acc.push_str(&s);
                        acc
                    })
                },
            )?,
        };

        Ok(Searchable { re, buffer })
    }
}
trait CliOptions {
    fn whole_word(&self) -> Self;
    fn whole_line(&self) -> Self;
    fn ignore_case(&self) -> Self;
    fn build_pattern(&self) -> Result<Regex, regex::Error>;
}

impl CliOptions for Cli {
    fn whole_word(&self) -> Self {
        self.pattern_modifier(self.whole_word, |pattern| {
            format!(r"\b{}\b", pattern)
        })
    }

    fn whole_line(&self) -> Self {
        self.pattern_modifier(self.whole_line, |pattern| {
            format!(r"^{}$", pattern)
        })
    }

    fn ignore_case(&self) -> Self {
        self.pattern_modifier(self.ignore_case, |pattern| {
            format!(r"(?i){}", pattern)
        })
    }

    fn build_pattern(&self) -> Result<Regex, regex::Error> {
        let pattern = self
            .whole_line()
            .whole_word()
            .ignore_case()
            .pattern;

        Regex::new(&pattern)
    }
}

pub struct Searchable {
    pub re: Regex,
    pub buffer: String,
}

impl Searchable {
    pub fn search(&self) -> impl Iterator<Item = Grepped> {
        self.buffer
            .lines()
            .enumerate()
            .filter(|(_, line)| self.re.is_match(line))
            .map(|(ln, line)| Grepped::new(ln + 1, line))
    }
}

#[derive(Debug)]
pub struct Grepped<'a> {
    line_number: usize,
    matched: &'a str,
}

impl<'a> Grepped<'a> {
    pub fn new(line_number: usize, matched: &'a str) -> Self {
        Self {
            line_number,
            matched,
        }
    }

    pub fn to_colored(&self, re: &Regex) -> String {
        format!(
            "{}: {}\n",
            self.line_number.to_string().green(),
            self.matched.paint_it_red(re)
        )
    }

    pub fn to_non_colored(&self) -> String {
        format!("{}: {}\n", self.line_number, self.matched)
    }
}

pub trait Colorize {
    fn red(self) -> String;
    fn green(self) -> String;
    fn paint_it_red(self, re: &Regex) -> String;
}

impl<'a> Colorize for &'a str {
    fn red(self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }

    fn green(self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }

    fn paint_it_red(self, re: &Regex) -> String {
        re.replace_all(self, |caps: &regex::Captures| caps[0].red())
            .to_string()
    }
}
