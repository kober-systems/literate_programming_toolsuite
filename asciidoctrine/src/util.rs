use std::fs;
use std::path::Path;
use std::io::{self, ErrorKind};
use std::collections::HashMap;

pub trait Environment {
  fn read_to_string(&mut self, path: &str) -> crate::Result<String>;
  fn write(&mut self, path: &str, content: &str) -> crate::Result<()>;
}

pub struct Io {}

impl Io {
  pub fn new() -> Self {
    Io {}
  }
}

impl Environment for Io {
  fn read_to_string(&mut self, path: &str) -> crate::Result<String> {
    Ok(fs::read_to_string(path)?)
  }

  fn write(&mut self, path: &str, content: &str) -> crate::Result<()> {
    let path = Path::new(path);
    if let Some(path) = path.parent() {
      if !path.exists() {
        fs::create_dir_all(path)?;
      }
    }

    if path.exists() {
      let old_content = fs::read_to_string(path)?;
      if old_content == content {
        return Ok(());
      }
    }
    fs::write(path, content)?;

    Ok(())
  }
}

pub struct Cache {
  files: HashMap<String, String>,
}

impl Cache {
  pub fn new() -> Self {
    Cache {
      files: HashMap::default(),
    }
  }

  pub fn get_files(self) -> HashMap<String, String> {
    self.files
  }
}

impl Environment for Cache {
  fn read_to_string(&mut self, path: &str) -> crate::Result<String> {
    Ok(
      self
        .files
        .remove(path)
        .ok_or(io::Error::new(ErrorKind::NotFound, "file not found in cache"))?
    )
  }

  fn write(&mut self, path: &str, content: &str) -> crate::Result<()> {
    self.files.insert(path.to_string(), content.to_string());

    Ok(())
  }
}

pub enum Env {
  Io(Io),
  Cache(Cache),
}

impl Env {
  pub fn get_cache(self) -> Option<HashMap<String, String>> {
    match self {
      Env::Io(_) => None,
      Env::Cache(env) => Some(env.get_files()),
    }
  }
}

impl Environment for Env {
  fn read_to_string(&mut self, path: &str) -> crate::Result<String> {
    match self {
      Env::Io(env) => env.read_to_string(path),
      Env::Cache(env) => env.read_to_string(path),
    }
  }

  fn write(&mut self, path: &str, content: &str) -> crate::Result<()> {
    match self {
      Env::Io(env) => env.write(path, content),
      Env::Cache(env) => env.write(path, content),
    }
  }
}
