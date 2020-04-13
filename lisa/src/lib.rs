extern crate asciidoctrine;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate topological_sort;
extern crate rlua;

mod codeblock_parser;

use asciidoctrine::*;
use std::collections::HashMap;
use topological_sort::TopologicalSort;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::io::Write;
use std::process::{Command, Stdio};
use rlua::Lua;

#[derive(Clone, Debug)]
pub enum SnippetType {
  Save(String),
  Eval(String),
  Pipe,
  Plain,
}

#[derive(Clone, Debug)]
pub struct Snippet {
  pub kind: SnippetType,
  pub content: String,
  pub children: Vec<Snippet>,
  /// List of all keys the snippet depends on
  /// before it can be processed
  pub depends_on: Vec<String>,
}
#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("a nessessary attribute is missing")]
  Missing,
  #[error("io problem")]
  Io(#[from] std::io::Error),
  #[error("Child process stdin has not been captured!")]
  Childprocess,
}


pub struct Lisa {
  snippets: HashMap<String, Snippet>,
  dependencies: TopologicalSort<String>,
}

impl Lisa {
  pub fn new() -> Self {
    Lisa {
      snippets: HashMap::default(),
      dependencies: TopologicalSort::new(),
    }
  }

  /// Gets recursively all snippets from an element
  pub fn extract(&mut self, input: &ElementSpan) -> Result<(), Error> {
    match &input.element {
      Element::TypedBlock {
        kind: BlockType::Listing,
      } => {
        let args = &mut input.positional_attributes.iter();
        if !(args.next() == Some(&AttributeValue::Ref("source"))) {
          return Ok(());
        }
        let mut interpreter = None;
        if let Some(value) = args.next()  {
          match &value {
            AttributeValue::Ref(value) => {
              interpreter = Some(value.to_string());
            },
            AttributeValue::String(value) => {
              interpreter = Some(value.clone());
            }
          }
        }
        let mut content = None;
        let mut path = None;
        let mut title = None;
        let mut id =
          "_id_".to_string() + &input.start.to_string() + &"_".to_string() + &input.end.to_string(); // TODO Vielleicht Datei + Zeile?
        for attribute in input.attributes.iter() {
          if attribute.key == "anchor" {
            id = match &attribute.value {
              AttributeValue::String(value) => value.clone(),
              AttributeValue::Ref(value) => value.to_string(),
            };
          }
          if attribute.key == "path" {
            path = match &attribute.value {
              AttributeValue::String(value) => Some(value.clone()),
              AttributeValue::Ref(value) => Some(value.to_string()),
            };
          }
          if attribute.key == "title" {
            title = match &attribute.value {
              AttributeValue::String(value) => Some(value.clone()),
              AttributeValue::Ref(value) => Some(value.to_string()),
            };
          }
          if attribute.key == "content" {
            content = match &attribute.value {
              AttributeValue::String(value) => Some(value.clone()),
              AttributeValue::Ref(value) => Some(value.to_string()),
            };
          }
          if attribute.key == "interpreter" {
            interpreter = match &attribute.value {
              AttributeValue::String(value) => Some(value.clone()),
              AttributeValue::Ref(value) => Some(value.to_string()),
            };
          }
        }
        if path == None {
          path = title;
        }
        let mut kind = SnippetType::Plain;

        for argument in args {
          match argument {
            AttributeValue::Ref("save") => {
              let path = match &path {
                Some(path) => path.clone(),
                None => id.clone(),
              };
              kind = SnippetType::Save(path);
            }
            AttributeValue::Ref("eval") => {
              let interpreter = interpreter.clone().ok_or(Error::Missing)?;
              kind = SnippetType::Eval(interpreter);
            }
            AttributeValue::Ref("pipe") => {
              kind = SnippetType::Pipe;
            }
            _ => (),
          }
        }
        if content == None {
          content = Some(input.content.to_string());
        }
        let content = match content {
          Some(content) => content,
          None => "".to_string(),
        };
        let mut dependencies = Vec::new();
        for dependency in codeblock_parser::get_dependencies(content.as_str()).iter() {
          dependencies.push(dependency.to_string());
        }
        self.store(
          id,
          Snippet {
            kind: kind,
            content: content,
            children: Vec::new(),
            depends_on: dependencies,
          },
        );
      }
      Element::IncludeElement(ast) => {
        self.extract_ast(&ast.inner);
      }
      _ => {
        for element in input.children.iter() {
          self.extract(element);
        }
      }
    }

    Ok(())
  }

  /// Stores a snippet in the internal database
  pub fn store(&mut self, name: String, snippet: Snippet) {
    let base = self.snippets.get_mut(&name);
    match base {
      Some(base) => {
        if &base.children.len() < &1 {
          let other = base.clone();
          &base.children.push(other);
        }
        base.content.push_str("\n");
        base.content.push_str(snippet.content.as_str());
        base.children.push(snippet);
      }
      None => {
        self.snippets.insert(name, snippet);
      }
    }
  }

  /// Builds the dependency tree for topological sorting
  pub fn check_dependencies(&mut self) {
    for (key, snippet) in self.snippets.iter() {
      // TODO Vielleicht sollten nur `save` und `eval` snippets
      // unabhängig von dependencies aufgenommen werden?
      self.dependencies.insert(key);

      for child in snippet.children.iter() {
        for dependency in child.depends_on.iter() {
          self.dependencies.add_dependency(dependency, key);
        }
      }
      for dependency in snippet.depends_on.iter() {
        self.dependencies.add_dependency(dependency, key);
      }
    }
  }

  /// Saves a Snippet to a file
  pub fn save(&self, path: String, content: String) -> Result<(), Error> {
    // TODO Allow directory prefix from options

    let path = Path::new(&path);
    if let Some(path) = path.parent() {
      if !path.exists() {
        fs::create_dir_all(path)?;
      }
    }

    let content = content.lines()
                         .map(|line| { String::from(line.trim_end()) + "\n" })
                         .collect::<String>();

    fs::write(path, content)?;

    Ok(())
  }

  /// Run a snippet in an interpreter
  pub fn eval(&self, interpreter: String, content: String) -> Result<(), Error> {

    let mut eval = Command::new(interpreter).stdin(Stdio::piped())
      .stderr(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()?;

    eval.stdin
      .as_mut()
      .ok_or(Error::Childprocess)?
      .write_all(content.as_bytes())?; // TODO Wie soll EOF gesendet werden?

    let output = eval.wait_with_output()?;

    // TODO in den Asciidoc AST einbinden
    if output.status.success() {
      let out = match String::from_utf8(output.stdout) {
        Ok(out) => out,
        Err(_) => "Error: Couldn't decode stdout".to_string(),
      };
      println!("{}", out); // TODO entfernen
    } else {
      let err = match String::from_utf8(output.stderr) {
        Ok(out) => out,
        Err(_) => "Error: Couldn't decode stderr".to_string(),
      };
      println!("External command failed:\n {}", err) // TODO entfernen
    }

    Ok(())
  }

  /// Gets all snippets from the ast
  pub fn extract_ast(&mut self, input: &AST) -> Result<(), Error> {
    // extract snippets from all inner elements
    for element in input.elements.iter() {
      self.extract(element);
    }

    Ok(())
  }

  /// Build all snippets (Runs the vm)
  pub fn generate_outputs(&mut self, mut ast: &AST) -> Result<(), Error> {
    loop {
      let key = self.dependencies.pop();
      match key {
        Some(key) => {
          let snippet = self.snippets.remove(&key.clone());

          match snippet {
            Some(mut snippet) => {
              let content = snippet.content.clone();

              let mut snippets = HashMap::new();
              for (key, snippet) in &self.snippets {
                snippets.insert(key.as_str(), snippet.content.as_str());
              }
              let content = codeblock_parser::merge_dependencies(content.as_str(), &snippets);
              snippet.content = content.clone();
              self.snippets.insert(key.clone(), snippet.clone());

              match &snippet.kind {
                SnippetType::Eval(interpreter) => {
                  self.eval(interpreter.to_string(), content)?;
                }
                SnippetType::Plain => {}
                SnippetType::Save(path) => {
                  let path = String::from_str(&path).unwrap();
                  self.save(path, content)?;
                }
                SnippetType::Pipe => {
                  let lua = Lua::new();

                  lua.context(|lua| -> rlua::Result<()> {

                    lua.load(&content).set_name(&key)?.exec()?;
                    Ok(())
                  });
                }
              }
            }
            None => {
              // TODO Fehlermeldung im AST. Ein Snippet sollte zu
              // diesem Zeitpunkt immer bereits erstellt sein.
              println!("Error: Dependency `{}` nicht gefunden", key);
            }
          }
        }
        None => {
          if !self.dependencies.is_empty() {
            println!(
              "Error: Es ist ein Ring in den Abhängigkeiten ({:#?})",
              self.dependencies
            );
          }
          break;
        }
      }
    }

    Ok(())
  }
}

impl Extension for Lisa {
  fn transform<'a>(&mut self, mut input: AST<'a>) -> AST<'a> {
    self.extract_ast(&input);

    // TODO Das Snippet Stack Program vorbereiten.
    self.check_dependencies();

    self.generate_outputs(&input);

    input
  }
}
