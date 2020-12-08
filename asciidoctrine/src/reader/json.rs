pub use crate::ast::*;
use crate::options::Opts;
use crate::Result;

pub struct JsonReader {}

impl JsonReader {
  pub fn new() -> Self {
    JsonReader {}
  }
}

impl crate::Reader for JsonReader {
  fn parse<'a>(&self, input: &'a str, _args: &Opts) -> Result<AST<'a>> {
    let ast = serde_json::from_str(input)?;

    Ok(ast)
  }
}
