use std::io;
use thiserror::Error;

mod ast;
pub use ast::*;
pub mod reader;

#[derive(Error, Debug)]
pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

pub trait Reader {
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>>;
}

pub trait Writer<T: io::Write> {
  fn write<'a>(&mut self, ast: AST, out: T) -> Result<()>;
}
