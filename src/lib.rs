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
    /// Case insensitivity
    #[arg(short, long, env = "IGNORE_CASE", default_value_t = false)]
    pub ignore_case: bool,
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

pub fn search<'a>(re: &Regex, contents: &'a str) -> Vec<Grepped<'a>> {
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| re.is_match(line))
        .map(|(ln, line)| Grepped::new(ln + 1, line))
        .collect()
}

pub fn stdout_print(arr: Vec<Grepped>, re: &Regex, test: bool) -> String {
    arr.iter()
        .map(|grepped| match test {
            false => grepped.to_colored(re),
            true => grepped.to_non_colored(),
        })
        .fold(String::new(), |mut acc, s| {
            acc.push_str(&s);
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_search() {
        let pattern = "foo";
        let re = Regex::new(pattern).unwrap();
        let contents = "        foo bar
        bar baz
        foo baz
";

        let results = search(&re, contents);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].matched, "        foo bar");
        assert_eq!(results[1].line_number, 3);
        assert_eq!(results[1].matched, "        foo baz");

        let pattern = "(?i)foo";
        let re = Regex::new(pattern).unwrap();
        let contents = "\
        Foo bar
        bar baz
        foo baz";

        let results = search(&re, contents);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].matched, "Foo bar");
        assert_eq!(results[1].line_number, 3);
        assert_eq!(results[1].matched, "        foo baz");

        let pattern = "xyz";
        let re = Regex::new(pattern).unwrap();
        let contents = "\
        foo bar
        bar baz
        foo baz";

        let results = search(&re, contents);
        assert!(results.is_empty());
    }

    #[test]
    fn test_stdout() {
        let pattern = "foo";
        let re = Regex::new(pattern).unwrap();
        let grepped = Grepped::new(1, "foo bar");

        let expected =
            format!("{}: {}\n", "1".green(), "foo bar".paint_it_red(&re));
        assert_eq!(grepped.to_colored(&re), expected);

        let expected = "1: foo bar\n".to_string();
        assert_eq!(grepped.to_non_colored(), expected);
    }
}
