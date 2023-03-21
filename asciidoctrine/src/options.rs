use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), String>
where
  T: std::str::FromStr,
  U: std::str::FromStr,
{
  let pos = s
    .find('=')
    .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
  let key = s[..pos]
    .parse()
    .or_else(|_| Err(format!("couldn't parse key in `{}`", s)))?;
  let value = s[pos + 1..]
    .parse()
    .or_else(|_| Err(format!("couldn't parse value in `{}`", s)))?;
  Ok((key, value))
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Reader {
  Asciidoc,
  Json,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Writer {
  Html5,
  Docbook,
  Pdf,
  Json,
  // The asciidoc output makes it possible
  // to use this tool as a preprocessor for
  // other asciidoc tools while it is maturing
  Asciidoc,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
  #[clap(short = 'r', long = "reader-format", default_value_t = Reader::Asciidoc)]
  #[clap(value_enum)]
  pub readerfmt: Reader,
  #[clap(short = 'w', long = "writer-format", default_value_t = Writer::Html5)]
  #[clap(value_enum)]
  pub writerfmt: Writer,
  #[clap(short = 'e', long = "extension")]
  pub extensions: Vec<String>,
  #[clap(long)]
  pub template: Option<PathBuf>,
  #[clap(long)]
  pub stylesheet: Option<PathBuf>,
  #[clap(short = 'a', long = "attribute")]
  #[clap(value_parser = parse_key_val::<String, String>, number_of_values = 1)]
  defines: Vec<(String, String)>,
  #[clap(name = "FILE")]
  pub input: Option<PathBuf>,
  #[clap(short = 'o')]
  pub output: Option<PathBuf>,
}

pub fn from_args() -> Opts {
  Opts::parse()
}
