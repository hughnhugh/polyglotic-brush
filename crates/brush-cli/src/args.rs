// Copyright (c) 2026 Polyglotic. All rights reserved.

//! The command line, read by hand: a command, then `--name value` pairs and
//! bare `--flags`.

#[derive(Debug)]
pub struct Args {
    rest: Vec<String>,
}

impl Args {
    pub fn from_env() -> Self {
        Self { rest: std::env::args().skip(1).collect() }
    }

    #[cfg(test)]
    pub fn of(words: &[&str]) -> Self {
        Self { rest: words.iter().map(|w| (*w).to_string()).collect() }
    }

    /// The first word, if it isn't an option.
    pub fn command(&mut self) -> Option<String> {
        match self.rest.first() {
            Some(first) if !first.starts_with("--") => Some(self.rest.remove(0)),
            _ => None,
        }
    }

    /// Whether `--name` was given (and take it).
    pub fn flag(&mut self, name: &str) -> bool {
        match self.rest.iter().position(|a| a == name) {
            Some(at) => {
                self.rest.remove(at);
                true
            }
            None => false,
        }
    }

    /// The word after `--name` (and take both).
    pub fn value(&mut self, name: &str) -> Option<String> {
        let at = self.rest.iter().position(|a| a == name)?;
        if at + 1 >= self.rest.len() {
            return None;
        }
        self.rest.remove(at);
        Some(self.rest.remove(at))
    }

    /// `--name 12`.
    pub fn number(&mut self, name: &str) -> Result<Option<u32>, String> {
        match self.value(name) {
            None => Ok(None),
            Some(word) => word.parse().map(Some).map_err(|_| format!("{name} wants a whole number, not '{word}'")),
        }
    }

    /// `--name 1,2,3`.
    pub fn numbers<T: std::str::FromStr>(&mut self, name: &str) -> Result<Option<Vec<T>>, String> {
        match self.value(name) {
            None => Ok(None),
            Some(word) => word
                .split(',')
                .map(|part| {
                    part.trim().parse::<T>().map_err(|_| format!("{name} wants numbers with commas, not '{word}'"))
                })
                .collect::<Result<Vec<T>, String>>()
                .map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn a_command_then_options_in_any_order() {
        let mut args = Args::of(&["bench", "--all", "--grids", "512, 1024", "--budget-ms", "200"]);
        assert_eq!(args.command().as_deref(), Some("bench"));
        assert_eq!(args.numbers::<u32>("--grids"), Ok(Some(vec![512, 1024])));
        assert!(args.flag("--all"));
        assert!(!args.flag("--all"));
        assert_eq!(args.number("--budget-ms"), Ok(Some(200)));
        assert_eq!(args.value("--out"), None);
    }

    #[test]
    fn a_bad_number_is_said_in_words() {
        let mut args = Args::of(&["--seconds", "ten", "--rect", "1,2,x,4"]);
        assert_eq!(args.command(), None);
        assert!(args.number("--seconds").unwrap_err().contains("'ten'"));
        assert!(args.numbers::<i32>("--rect").is_err());
    }
}
