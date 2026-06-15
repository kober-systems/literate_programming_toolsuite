use std::io::Write;

use crate::{Writer, AST};

pub struct JsonWriter;

impl<T: Write> Writer<T> for JsonWriter {
  fn write<'a>(&mut self, ast: AST<'a>, out: T) -> crate::Result<()> {
    serde_json::to_writer_pretty(out, &ast)
      .map_err(|e| crate::Error::ParseError(format!("Failed to write JSON: {}", e)))
  }
}
