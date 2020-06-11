extern crate asciidoctrine;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate topological_sort;
extern crate rhai;

mod codeblock_parser;

use asciidoctrine::*;
use std::collections::HashMap;
use std::collections::hash_map;
use topological_sort::TopologicalSort;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::io::Write;
use std::process::{Command, Stdio};
use rhai::RegisterFn;
use core::cell::RefCell;
use std::rc::Rc;

pub struct SnippetDB {
  snippets: HashMap<String, Snippet>,
}

impl SnippetDB {
  pub fn new() -> Self {
    SnippetDB {
      snippets: HashMap::default(),
    }
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
        base.content.push_str(base.join_str.as_str());
        base.content.push_str(snippet.content.as_str());
        base.children.push(snippet);
      }
      None => {
        self.snippets.insert(name, snippet);
      }
    }
  }

  pub fn get(&self, name: &str) -> Option<&Snippet> {
    self.snippets.get(name)
  }

  pub fn pop(&mut self, name: &str) -> Option<Snippet> {
    self.snippets.remove(name)
  }

  pub fn iter(&self) -> hash_map::Iter<String, Snippet> {
    self.snippets.iter()
  }
}

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
  pub raw: bool,
  pub join_str: String,
}

#[derive(Clone)]
struct LisaWrapper {
  pub snippets: Rc<RefCell<SnippetDB>>,
}

impl LisaWrapper {
  pub fn store(&mut self, name: String, content: String) {
    let mut snippets = self.snippets.borrow_mut();

    snippets.pop(&name);

    snippets.store(name, Snippet {
      kind: SnippetType::Plain,
      content: content,
      children: Vec::new(),
      depends_on: Vec::new(),
      raw: true,
      join_str: "".to_string(),
    });
  }

  pub fn get_snippet(&mut self, name: String) -> String {
    let snippets = self.snippets.borrow_mut();

    match snippets.get(&name) {
      Some(snippet) => snippet.content.clone(),
      None => "".to_string(),
    }
  }
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
  dependencies: TopologicalSort<String>,
}

impl Lisa {
  pub fn new() -> Self {
    Lisa {
      dependencies: TopologicalSort::new(),
    }
  }

  /// Gets recursively all snippets from an element
  pub fn extract(&mut self, mut snippets: SnippetDB, input: &ElementSpan) -> SnippetDB {
    match &input.element {
      Element::TypedBlock {
        kind: BlockType::Listing,
      } => {
        let args = &mut input.positional_attributes.iter();
        if !(args.next() == Some(&AttributeValue::Ref("source"))) {
          return snippets;
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
        let mut raw = false;
        let join_str = input.get_attribute("lisa-join").unwrap_or("\n\n".to_string());
        let join_str = join_str.replace("\\\\", "\\").replace("\\n", "\n").replace("\\t", "\t");

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
              let interpreter = interpreter.clone().unwrap_or("interpreter_missing".to_string());
              kind = SnippetType::Eval(interpreter);
            }
            AttributeValue::Ref("pipe") => {
              kind = SnippetType::Pipe;
            }
            AttributeValue::Ref("lisa-raw") => {
              raw = true;
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
        snippets.store(
          id,
          Snippet {
            kind: kind,
            content: content,
            children: Vec::new(),
            depends_on: dependencies,
            raw: raw,
            join_str: join_str,
          },
        );

        snippets
      }
      Element::IncludeElement(ast) => {
        ast.inner.elements.iter().fold(snippets, |snippets, element| {
          self.extract(snippets, element)
        })
      }
      _ => {
        input.children.iter().fold(snippets, |snippets, element| {
          self.extract(snippets, element)
        })
      }
    }
  }

  /// Builds the dependency tree for topological sorting
  pub fn calculate_snippet_ordering(&mut self, snippets: &SnippetDB) {
    for (key, snippet) in snippets.iter() {
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

    let old_content = fs::read_to_string(path)?;
    if old_content == content {
      return Ok(());
    }

    fs::write(path, content)?;

    Ok(())
  }

  /// Run a snippet in an interpreter
  pub fn eval(&self, interpreter: String, content: String) -> Result<(), Error> {

    let mut eval = Command::new(interpreter)
      .stdin(Stdio::piped())
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
  pub fn extract_ast(&mut self, input: &AST) -> Result<SnippetDB, Error> {
    let snippets = SnippetDB::new();

    // extract snippets from all inner elements
    let snippets = input.elements.iter().fold(snippets, |snippets, element| {
      self.extract(snippets, element)
    });

    Ok(snippets)
  }

  /// Build all snippets (Runs the vm)
  pub fn generate_outputs(&mut self, snippets: SnippetDB, mut ast: &AST) -> Result<(), Error> {
    let db = Rc::new(RefCell::new(snippets));
    let snippets = Rc::clone(&db);

    loop {
      let key = self.dependencies.pop();
      let snippet = match key {
        Some(key) => {
          let mut snippets = snippets.borrow_mut();
          let snippet = snippets.pop(&key.clone());

          match snippet {
            Some(mut snippet) => {
              if !snippet.raw {
                let content = snippet.content.clone();
                let content = codeblock_parser::merge_dependencies(content.as_str(), &snippets);
                snippet.content = content.clone();
              };

              snippets.store(key, snippet.clone());
              Some(snippet)
            }
            None => {
              // TODO Fehlermeldung im AST. Ein Snippet sollte zu
              // diesem Zeitpunkt immer bereits erstellt sein.
              println!("Error: Dependency `{}` nicht gefunden", key);
              None
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
      };

      if let Some(snippet) = snippet {
        match &snippet.kind {
          SnippetType::Eval(interpreter) => {
            self.eval(interpreter.to_string(), snippet.content)?;
          }
          SnippetType::Plain => {}
          SnippetType::Save(path) => {
            let path = String::from_str(&path).unwrap();
            self.save(path, snippet.content)?;
          }
          SnippetType::Pipe => {
            let mut engine = rhai::Engine::new();

            let mut scope = rhai::Scope::new();

            let wrapper = LisaWrapper { snippets: Rc::clone(&db) };
            scope.push_constant("lisa", wrapper);

            engine.register_type_with_name::<LisaWrapper>("LisaType");
            engine.register_fn("store", LisaWrapper::store);
            engine.register_fn("get_snippet", LisaWrapper::get_snippet);

            engine.eval_with_scope::<()>(&mut scope, &snippet.content);
          }
        }
      }
    }

    Ok(())
  }
}

impl Extension for Lisa {
  fn transform<'a>(&mut self, mut input: AST<'a>) -> AST<'a> {
    let snippets = self.extract_ast(&input).unwrap();

    self.calculate_snippet_ordering(&snippets);

    self.generate_outputs(snippets, &input);

    input
  }
}
