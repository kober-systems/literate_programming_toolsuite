extern crate clap;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub use structopt::StructOpt;

mod ast;
pub use ast::*;
pub mod options;
mod reader;

pub trait Reader {
  fn parse(&self, input: &str) -> AST;
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
pub fn parse_ast(input: &str) -> AST {
  reader::asciidoc::parse_ast(input)
}
