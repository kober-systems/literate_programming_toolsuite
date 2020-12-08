use pest::Parser;

use crate::*;

#[derive(Parser, Debug)]
#[grammar = "codeblock.pest"]
pub struct CodeblockParser;

fn extract_identifier<'a>(element: &pest::iterators::Pair<'a, codeblock_parser::Rule>) -> &'a str {
  match element.as_rule() {
    Rule::reference => element.clone().into_inner().next().unwrap().as_str(),
    Rule::indented_reference => {
      let mut output = "";
      for element in element.clone().into_inner() {
        match element.as_rule() {
          Rule::reference => {
            output = element.into_inner().next().unwrap().as_str();
            break;
          }
          _ => (),
        }
      }
      output
    }
    _ => "",
  }
}

fn extract_join_str<'a>(element: &pest::iterators::Pair<'a, codeblock_parser::Rule>) -> &'a str {
  match element.as_rule() {
    Rule::reference => {
      match element.clone().into_inner()
        .find(|element| {
          match element.as_rule() {
            Rule::attributes => true,
            _ => false
          }
        }) {
        Some(element) => {
          extract_join_str(&element)
        }
        None => "\n"
      }
    }
    Rule::attributes => {
      match element.clone().into_inner()
        .find(|element| {
          match element.as_rule() {
            Rule::attribute => {
              let mut attribute = element.clone().into_inner();
              let key = attribute.next().unwrap();

              key.as_str() == "join"
            },
            _ => false
          }
        }) {
        Some(element) => {
          let mut attribute = element.clone().into_inner();
          attribute.next();
          let value = attribute.next().unwrap();

          value.as_str()
        }
        None => "\n"
      }
    }
    Rule::indented_reference => {
      match element.clone().into_inner()
        .find(|element| {
          match element.as_rule() {
            Rule::reference => true,
            _ => false,
          }
        }) {
        Some(element) => {
          extract_join_str(&element)
        }
        None => "\n"
      }
    }
    _ => "\n",
  }
}

fn extract_indentation<'a>(element: &pest::iterators::Pair<'a, codeblock_parser::Rule>) -> &'a str {
  let mut output = "";
  for element in element.clone().into_inner() {
    match element.as_rule() {
      Rule::indentation => {
        output = element.as_str();
        break;
      }
      _ => (),
    }
  }
  output
}

/// Extracts the ids of used snippets from a depending snippet
pub fn get_dependencies(input: &str) -> Vec<&str> {
  let mut depends_on_ids = Vec::new();

  let ast = CodeblockParser::parse(Rule::codeblock, input).expect("couldn't parse input.");

  for element in ast {
    match element.as_rule() {
      Rule::reference => {
        depends_on_ids.push(extract_identifier(&element));
      }
      Rule::indented_reference => {
        depends_on_ids.push(extract_identifier(&element));
      }
      _ => (),
    }
  }

  depends_on_ids
}

/// Merges the snippets into the depending snippet
pub fn merge_dependencies(input: &str, snippets: &SnippetDB) -> String {
  let mut output = String::new();

  let ast = CodeblockParser::parse(Rule::codeblock, input).expect("couldn't parse input.");

  for element in ast {
    match element.as_rule() {
      Rule::reference => {
        let identifier = extract_identifier(&element);
        let join_str = extract_join_str(&element)
          .replace("\\n", "\n");
        // TODO Den passenden snippet suchen
        let snippet = snippets.get(identifier);
        // TODO Den snippet einfügen
        match snippet {
          Some(snippet) => {
            output.push_str(&snippet.get_content(&join_str));
          }
          None => {
            // TODO Fehlermeldung? Müsste vorher bereits abgefangen sein.
          }
        }
      }
      Rule::indented_reference => {
        let identifier = extract_identifier(&element);
        let join_str = extract_join_str(&element)
          .replace("\\n", "\n");
        let indentation = extract_indentation(&element);
        // TODO Den passenden snippet suchen
        let snippet = snippets.get(identifier);
        // TODO Den snippet einfügen und indentation beruecksichtigen
        match snippet {
          Some(snippet) => {
            for line in snippet.get_content(&join_str).lines() {
              output.push_str("\n");
              output.push_str(indentation);
              output.push_str(line);
            }
          }
          None => {
            // TODO Fehlermeldung? Müsste vorher bereits abgefangen sein.
          }
        }
      }
      Rule::code => {
        output.push_str(element.as_str());
      }
      _ => (),
    }
  }
  output
}
