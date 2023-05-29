use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub trait Environment {
  fn read_to_string(&mut self, path: &str) -> crate::Result<String>;
  fn write(&mut self, path: &str, content: &str) -> crate::Result<()>;
  fn eval(&mut self, interpreter: &str, content: &str) -> crate::Result<(bool, String, String)>; // success, Stdout, Stderr
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

  fn eval(&mut self, interpreter: &str, content: &str) -> crate::Result<(bool, String, String)> {
    let mut eval = Command::new(interpreter)
      .stdin(Stdio::piped())
      .stderr(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()?;

    eval
      .stdin
      .as_mut()
      .ok_or(crate::AsciidoctrineError::Childprocess)?
      .write_all(content.as_bytes())?; // TODO Wie soll EOF gesendet werden?
    let output = eval.wait_with_output()?;

    let success = output.status.success();
    let out = match String::from_utf8(output.stdout) {
      Ok(out) => out,
      Err(_) => "Error: Couldn't decode stdout".to_string(),
    };
    let err = match String::from_utf8(output.stderr) {
      Ok(out) => out,
      Err(_) => "Error: Couldn't decode stderr".to_string(),
    };

    Ok((success, out, err))
  }
}

pub struct Cache {
  files: HashMap<String, String>,
  evaluations: HashMap<(String, String),(
    bool,
    String,
    String,
    Vec<(String, String)>,
    Vec<String>)>,
}

impl Cache {
  pub fn new() -> Self {
    Cache {
      files: HashMap::default(),
      evaluations: HashMap::default(),
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

  fn eval(&mut self, interpreter: &str, content: &str) -> crate::Result<(bool, String, String)> {
    match self.evaluations.remove(
      &(interpreter.to_string(), content.to_string()))
    {
      Some((success, out, err, add, remove)) => {
        for path in remove.iter() {
          self.files.remove(path);
        }
        for (path, content) in add.into_iter() {
          self.files.insert(path, content);
        }
        Ok((success, out, err))
      }
      None => Err(crate::AsciidoctrineError::Childprocess)
    }
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

  fn eval(&mut self, interpreter: &str, content: &str) -> crate::Result<(bool, String, String)> {
    match self {
      Env::Io(env) => env.eval(interpreter, content),
      Env::Cache(env) => env.eval(interpreter, content),
    }
  }
}
