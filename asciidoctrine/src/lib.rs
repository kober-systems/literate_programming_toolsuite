extern crate clap;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub use structopt::StructOpt;
use thiserror::Error;

mod ast;
pub use ast::*;
pub mod options;
mod reader;
pub use reader::asciidoc::AsciidocReader;

#[derive(Error, Debug)]
pub enum AsciidoctrineError {
  #[error("could not parse input")]
  Parse(String),
}

type Result<T> = std::result::Result<T, AsciidoctrineError>;

pub trait Reader {
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>>;
}

pub trait Extension {
  // TODO Result zuürckgeben
  // TODO Options (Kann auch über Attributes in AST gemacht werden)
  fn transform<'a>(&mut self, input: AST<'a>) -> AST<'a>;
}

pub trait Writer {
  // TODO Result zurückgeben mit Fehler oder Liste der Geschriebenen Dateien
  // TODO Options
  fn write(&self, ast: AST);
}

// TODO Add Options
pub fn parse_ast(input: &str) -> Result<AST> {
  reader::asciidoc::parse_ast(input)
}
