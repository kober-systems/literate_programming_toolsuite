pub use crate::ast::*;
use crate::{options, Result};
use std::io;

pub struct JsonWriter {}

impl JsonWriter {
  pub fn new() -> Self {
    JsonWriter {}
  }
}

impl<T: io::Write> crate::Writer<T> for JsonWriter {
  fn write<'a>(&mut self, ast: AST, _args: &options::Opts, mut out: T) -> Result<()> {
    out.write_all(serde_json::to_string_pretty(&ast)?.as_bytes())?;
    out.flush()?;

    Ok(())
  }
}
