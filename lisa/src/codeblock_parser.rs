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
        // TODO Den passenden snippet suchen
        let snippet = snippets.get(identifier);
        // TODO Den snippet einfügen
        match snippet {
          Some(snippet) => {
            output.push_str(&snippet.content);
          }
          None => {
            // TODO Fehlermeldung? Müsste vorher bereits abgefangen sein.
          }
        }
      }
      Rule::indented_reference => {
        let identifier = extract_identifier(&element);
        let indentation = extract_indentation(&element);
        // TODO Den passenden snippet suchen
        let snippet = snippets.get(identifier);
        // TODO Den snippet einfügen und indentation beruecksichtigen
        match snippet {
          Some(snippet) => {
            for line in snippet.content.lines() {
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
