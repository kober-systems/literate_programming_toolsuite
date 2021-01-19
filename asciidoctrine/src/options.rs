extern crate clap;

use clap::arg_enum;
use std::error::Error;
use std::path::PathBuf;
pub use structopt::StructOpt;

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
  T: std::str::FromStr,
  T::Err: Error + 'static,
  U: std::str::FromStr,
  U::Err: Error + 'static,
{
  let pos = s
    .find('=')
    .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
  Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

arg_enum! {
  #[derive(StructOpt, Debug)]
  pub enum Reader {
    Asciidoc,
    Json,
  }
}

arg_enum! {
  #[derive(StructOpt, Debug)]
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
}

#[derive(StructOpt, Debug)]
#[structopt(name = "asciidoctrine")]
pub struct Opts {
  #[structopt(short = "r", long = "reader-format", default_value = "asciidoc")]
  #[structopt(possible_values = &Reader::variants(), case_insensitive = true)]
  pub readerfmt: Reader,
  #[structopt(short = "w", long = "writer-format", default_value = "html5")]
  #[structopt(possible_values = &Writer::variants(), case_insensitive = true)]
  pub writerfmt: Writer,
  #[structopt(short = "e", long = "extension")]
  pub extensions: Vec<String>,
  #[structopt(long, parse(from_os_str))]
  pub template: Option<PathBuf>,
  #[structopt(long, parse(from_os_str))]
  pub stylesheet: Option<PathBuf>,
  #[structopt(short = "a", long = "attribute", parse(try_from_str = parse_key_val), number_of_values = 1)]
  defines: Vec<(String, String)>,
  #[structopt(name = "FILE", parse(from_os_str))]
  pub input: Option<PathBuf>,
  #[structopt(short = "o", parse(from_os_str))]
  pub output: Option<PathBuf>,
}

pub fn from_args() -> Opts {
  Opts::from_args()
}
