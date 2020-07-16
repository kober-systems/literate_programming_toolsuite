pub use crate::ast::*;
use crate::Result;
use std::io;

pub struct JsonWriter {}

impl JsonWriter {
  pub fn new() -> Self {
    JsonWriter {}
  }
}

impl<T: io::Write> crate::Writer<T> for JsonWriter {
  fn write<'a>(&self, ast: AST, mut out: T) -> Result<()> {
    out.write_all(serde_json::to_string_pretty(&ast)?.as_bytes())?;
    out.flush()?;

    Ok(())
  }
}
