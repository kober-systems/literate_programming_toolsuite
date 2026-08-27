use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn read_example(name: &str) -> Result<String> {
  Ok(fs::read_to_string(
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/examples/sequence-diagram")
      .join(name),
  )?)
}

