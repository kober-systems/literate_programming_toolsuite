extern crate clap;
extern crate serde;
extern crate serde_json;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::io;
pub use structopt::StructOpt;
use thiserror::Error;

mod ast;
pub use ast::*;
pub mod options;
pub mod util;
pub mod reader;
pub use reader::asciidoc::AsciidocReader;
pub use reader::json::JsonReader;
mod writer;
pub use writer::html::HtmlWriter;
pub use writer::json::JsonWriter;

#[derive(Error, Debug)]
pub enum AsciidoctrineError {
  #[error("could not parse input")]
  Parse(#[from] pest::error::Error<reader::asciidoc::Rule>),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, AsciidoctrineError>;

pub trait Reader {
  fn parse<'a>(&self, input: &'a str, args: &options::Opts) -> Result<AST<'a>>;
}

pub trait Extension {
  // TODO Options (Kann auch über Attributes in AST gemacht werden)
  fn transform<'a>(&mut self, input: AST<'a>) -> anyhow::Result<AST<'a>>;
}

pub trait Writer<T: io::Write> {
  // TODO Result zurückgeben mit Fehler oder Liste der Geschriebenen Dateien
  // TODO Options
  fn write<'a>(&self, ast: AST, out: T) -> Result<()>;
}

