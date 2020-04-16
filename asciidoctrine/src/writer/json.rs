pub use crate::ast::*;
use crate::writer::*;
use crate::Result;

pub struct JsonWriter {}

impl JsonWriter {
  pub fn new() -> Self {
    JsonWriter {}
  }
}

impl crate::Writer for JsonWriter {
  fn write<'a>(&self, ast: AST) -> Result<()> {
    let out = serde_json::to_string_pretty(&ast)?;
    print!("{}", out);

    Ok(())
  }
}
