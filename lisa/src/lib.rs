#[macro_use]
extern crate pest_derive;

mod codeblock_parser;

use asciidoctrine::*;
use std::collections::HashMap;
use std::collections::hash_map;
use topological_sort::TopologicalSort;
use core::cell::RefCell;
use std::rc::Rc;
use asciidoctrine::util::Environment;
#[macro_use]
extern crate log;

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
        for dependency in snippet.depends_on.clone().into_iter() {
          base.depends_on.push(dependency);
        }
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
  pub raw_content: String,
  pub children: Vec<Snippet>,
  /// List of all keys the snippet depends on
  /// before it can be processed
  pub depends_on: Vec<String>,
  pub attributes: HashMap<String, String>,
  pub raw: bool,
}

impl Snippet {
  fn get_content(&self, join_str: &str) -> String {
    if self.children.len() > 0 {
      let mut iter = self.children.iter();
      let start = iter.next().unwrap().content.clone();
      iter.fold(start, |mut base, snippet| {
        base.push_str(join_str);
        base.push_str(&snippet.content);
        base
      })
    } else {
      self.content.to_string()
    }
  }

  fn get_raw_content(&self, join_str: &str) -> String {
    if self.children.len() > 0 {
      let mut iter = self.children.iter();
      let start = iter.next().unwrap().raw_content.clone();
      iter.fold(start, |mut base, snippet| {
        base.push_str(join_str);
        base.push_str(&snippet.raw_content);
        base
      })
    } else {
      self.raw_content.to_string()
    }
  }
}

#[derive(Clone)]
struct LisaWrapper {
  pub snippets: Rc<RefCell<SnippetDB>>,
}

impl LisaWrapper {
  pub fn store(&mut self, name: &str, content: &str) {
    let mut snippets = self.snippets.borrow_mut();

    snippets.pop(name);

    snippets.store(
      name.to_string(),
      Snippet {
        kind: SnippetType::Plain,
        content: content.to_string(),
        raw_content: content.to_string(),
        children: Vec::new(),
        depends_on: Vec::new(),
        attributes: HashMap::default(),
        raw: true,
      },
    );
  }

  pub fn get_snippet(&mut self, name: &str) -> rhai::Dynamic {
    let snippets = self.snippets.borrow_mut();

    match snippets.get(name) {
      Some(snippet) => {
        let mut attributes: HashMap<rhai::ImmutableString, rhai::Dynamic> = HashMap::default();
        for (k,v) in snippet.attributes.clone().drain() {
          attributes.insert(k.into(), v.into());
        }

        let mut out: HashMap<rhai::ImmutableString, rhai::Dynamic> = HashMap::default();
        out.insert("content".into(), snippet.content.clone().into());
        out.insert("attrs".into(), attributes.into());

        out.into()
      },
      None => rhai::Dynamic::from(()),
    }
  }

  pub fn get_snippet_names(&mut self) -> rhai::Array {
    let mut snippets = self.snippets.borrow_mut();

    let mut out = rhai::Array::new();

    let mut keys = snippets
      .iter()
      .map(|(key, _)| { key.to_string() })
      .collect::<Vec<_>>();
    keys.sort();
    let out: rhai::Array = keys
      .into_iter()
      .map(|key| { key.into() })
      .collect();

    out
  }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("a nessessary attribute is missing")]
  Missing,
  #[error(transparent)]
  Asciidoctrine(#[from] asciidoctrine::AsciidoctrineError),
  #[error("io problem")]
  Io(#[from] std::io::Error),
}

pub struct Lisa {
  dependencies: TopologicalSort<String>,
  env: asciidoctrine::util::Env,
}

impl Lisa {
  pub fn new() -> Self {
    Lisa {
      dependencies: TopologicalSort::new(),
      env: util::Env::Io(util::Io::new()),
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
              interpreter = Some(*value);
            },
            AttributeValue::String(value) => {
              interpreter = Some(value.as_str());
            }
          }
        }

        let title = input.get_attribute("title");
        let path = input.get_attribute("path").or(title);

        let id = input.get_attribute("anchor").unwrap_or(
          &format!("_id_{}_{}", input.start, input.end),
        ).to_string(); // TODO Vielleicht Datei + Zeile?

        let interpreter = input.get_attribute("interpreter").or(interpreter);
        let mut raw = false;

        let mut kind = SnippetType::Plain;

        for argument in args {
          match argument {
            AttributeValue::Ref("save") => {
              let path = path.unwrap_or(id.as_str());
              kind = SnippetType::Save(path.to_string());
            }
            AttributeValue::Ref("eval") => {
              let interpreter = interpreter.clone().unwrap_or("interpreter missing");
              kind = SnippetType::Eval(interpreter.to_string());
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

        let mut attributes: HashMap<String, String> = HashMap::default();

        for key in input.attributes.iter().map(|attr|{ attr.key.clone() }) {
          attributes.insert(key.clone(), input.get_attribute(&key).unwrap().to_string());
        }

        let content = input
          .get_attribute("content")
          .unwrap_or(input.content);
        let mut dependencies = Vec::new();
        for dependency in codeblock_parser::get_dependencies(content).iter() {
          dependencies.push(dependency.to_string());
        }
        snippets.store(
          id.to_string(),
          Snippet {
            kind,
            content: content.to_string(),
            raw_content: content.to_string(),
            children: Vec::new(),
            depends_on: dependencies,
            attributes,
            raw,
          },
        );

        snippets
      }
      Element::Styled => {
        let id = match input.get_attribute("anchor") {
          Some(id) => id.to_string(),
          None => { return snippets; },
        };
        let kind = SnippetType::Plain;
        let raw = false;
        let dependencies = Vec::new();
        let mut attributes: HashMap<String, String> = HashMap::default();

        for key in input.attributes.iter().map(|attr|{ attr.key.clone() }) {
          attributes.insert(key.clone(), input.get_attribute(&key).unwrap().to_string());
        }
        let content = input
          .get_attribute("content")
          .unwrap_or(input.content);
        snippets.store(
          id.to_string(),
          Snippet {
            kind,
            content: content.to_string(),
            raw_content: content.to_string(),
            children: Vec::new(),
            depends_on: dependencies,
            attributes,
            raw,
          },
        );

        snippets
      }
      Element::IncludeElement(ast) => ast
        .inner
        .elements
        .iter()
        .fold(snippets, |snippets, element| {
          self.extract(snippets, element)
        }),
      _ => input.children.iter().fold(snippets, |snippets, element| {
        self.extract(snippets, element)
      }),
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
  pub fn save(&mut self, path: &str, content: &str) -> Result<(), Error> {
    let content = content.lines()
                         .map(|line| { String::from(line.trim_end()) + "\n" })
                         .collect::<String>();

    // TODO Allow directory prefix from options

    self.env.write(path, &content)?;

    Ok(())
  }

  /// Run a snippet in an interpreter
  pub fn eval(&mut self, interpreter: String, content: String) -> Result<(), Error> {

    let (success, out, err) = self.env.eval(&interpreter, &content)?;

    // TODO in den Asciidoc AST einbinden
    if success {
      info!("{}", out); // TODO entfernen
    } else {
      error!("External command failed:\n {}", err) // TODO entfernen
    }

    Ok(())
  }

  pub fn from_env(env: util::Env) -> Self {
    let mut base = Lisa::new();
    base.env = env;

    base
  }

  pub fn into_cache(self) -> Option<HashMap<String, String>> {
    self.env.get_cache()
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
    let source = ast.get_attribute("source").unwrap_or("");
    let db = Rc::new(RefCell::new(snippets));
    let snippets = Rc::clone(&db);

    loop {
      let key = self.dependencies.pop();
      let snippet = match &key {
        Some(key) => {
          let mut snippets = snippets.borrow_mut();
          let snippet = snippets.pop(&key);

          match snippet {
            Some(mut snippet) => {
              if !snippet.raw {
                if snippet.children.len() > 0 {
                  let mut children = Vec::new();
                  for mut child in snippet.children.clone().into_iter() {
                    let content = child.content.clone();
                    let content = codeblock_parser::merge_dependencies(content.as_str(), &snippets);
                    child.content = content;
                    children.push(child);
                  }
                  snippet.children = children;
                } else {
                  let content = snippet.content.clone();
                  let content = codeblock_parser::merge_dependencies(content.as_str(), &snippets);
                  snippet.content = content;
                }
              };

              snippets.store(key.to_string(), snippet.clone());
              Some(snippet)
            }
            None => {
              // TODO Fehlermeldung im AST. Ein Snippet sollte zu
              // diesem Zeitpunkt immer bereits erstellt sein.
              warn!("{}: Dependency `{}` nicht gefunden", source, key);
              None
            }
          }
        }
        None => {
          if !self.dependencies.is_empty() {
            error!(
              "Es ist ein Ring in den Abhängigkeiten ({:#?})",
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
            self.save(path, &snippet.content)?;
          }
          SnippetType::Pipe => {
            let mut engine = rhai::Engine::new();

            let mut scope = rhai::Scope::new();

            let wrapper = LisaWrapper {
              snippets: Rc::clone(&db)
            };
            scope.push_constant("lisa", wrapper);

            engine.register_type_with_name::<LisaWrapper>("LisaType");
            engine.register_fn("store", LisaWrapper::store);
            engine.register_fn("get_snippet", LisaWrapper::get_snippet);
            engine.register_fn("get_snippet_names", LisaWrapper::get_snippet_names);

            engine.eval_with_scope::<()>(&mut scope, &snippet.content)
              .unwrap_or_else(|e| {
                error!("Piping of snippet failed:\n {}", e);
              });
          }
        }
      }
    }

    Ok(())
  }
}

impl Extension for Lisa {
  fn transform<'a>(&mut self, input: AST<'a>) -> anyhow::Result<AST<'a>> {
    let snippets = self.extract_ast(&input)?;

    self.calculate_snippet_ordering(&snippets);

    self.generate_outputs(snippets, &input)?;

    Ok(input)
  }
}
