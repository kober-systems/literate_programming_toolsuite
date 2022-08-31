use pest::Parser;

use crate::*;

#[derive(Parser, Debug)]
#[grammar = "codeblock.pest"]
pub struct CodeblockParser;

#[derive(Clone)]
enum ReferenceParam {
  Value(String),
  Reference(String),
}

fn merge_dependencies_inner<'a>(
  ast: pest::iterators::Pairs<'a, codeblock_parser::Rule>,
  snippets: &SnippetDB,
  snippet_params: HashMap<String, ReferenceParam>
) -> String {

  let mut output = String::new();

  for element in ast {
    match element.as_rule() {
      Rule::reference => {
        let identifier = extract_identifier(&element);
        let join_str = extract_join_str(&element)
          .replace("\\n", "\n");
        match snippet_params.get(identifier) {
          Some(param) => match param {
            ReferenceParam::Value(param) => output.push_str(&param),
            ReferenceParam::Reference(param) => {
              let ast = CodeblockParser::parse(Rule::codeblock, &param).expect("couldn't parse input.");
              let mut snippet_params = snippet_params.clone();
              for element in ast
                .clone()
                .filter(|element| match element.as_rule() {
                  Rule::reference => true,
                  Rule::indented_reference => true,
                  _ => false
                })
                .flat_map(|element| element.clone().into_inner())
                .filter(|element| {
                  match element.as_rule() {
                    Rule::attributes => true,
                    _ => false
                  }
                })
                .flat_map(|element| element.clone().into_inner())
                .filter(|element| match element.as_rule() {
                  Rule::attribute_param => true,
                  _ => false
                })
              {
                let key = element
                  .clone()
                  .into_inner()
                  .find(|element| match element.as_rule() {
                    Rule::identifier => true,
                    _ => false
                  })
                  .map(|element| element.as_str().to_string())
                  .unwrap_or("".to_string());
                let value = element
                  .into_inner()
                  .find(|element| match element.as_rule() {
                    Rule::value => true,
                    Rule::reference => true,
                    _ => false
                  })
                  .map(|element| match element.as_rule() {
                    Rule::value => ReferenceParam::Value(element.as_str().to_string()),
                    Rule::reference => ReferenceParam::Reference(element.as_str().to_string()),
                    _ => ReferenceParam::Value("".to_string())
                  })
                  .unwrap();
                snippet_params.insert(key, value);

              }
              let content = merge_dependencies_inner(ast, snippets, snippet_params);
              output.push_str(&content);
            }
          }
          None => match snippets.get(identifier) {
            Some(snippet) => {
              if snippet_params.is_empty() {
                let content = snippet.get_content(&join_str);
                output.push_str(&content);
              } else {
                let input = snippet.get_raw_content(&join_str);
                let ast = CodeblockParser::parse(Rule::codeblock, &input).expect("couldn't parse input.");
                let mut snippet_params = snippet_params.clone();
                for element in ast
                  .clone()
                  .filter(|element| match element.as_rule() {
                    Rule::reference => true,
                    Rule::indented_reference => true,
                    _ => false
                  })
                  .flat_map(|element| element.clone().into_inner())
                  .filter(|element| {
                    match element.as_rule() {
                      Rule::attributes => true,
                      _ => false
                    }
                  })
                  .flat_map(|element| element.clone().into_inner())
                  .filter(|element| match element.as_rule() {
                    Rule::attribute_param => true,
                    _ => false
                  })
                {
                  let key = element
                    .clone()
                    .into_inner()
                    .find(|element| match element.as_rule() {
                      Rule::identifier => true,
                      _ => false
                    })
                    .map(|element| element.as_str().to_string())
                    .unwrap_or("".to_string());
                  let value = element
                    .into_inner()
                    .find(|element| match element.as_rule() {
                      Rule::value => true,
                      Rule::reference => true,
                      _ => false
                    })
                    .map(|element| match element.as_rule() {
                      Rule::value => ReferenceParam::Value(element.as_str().to_string()),
                      Rule::reference => ReferenceParam::Reference(element.as_str().to_string()),
                      _ => ReferenceParam::Value("".to_string())
                    })
                    .unwrap();
                  snippet_params.insert(key, value);

                }
                let content = merge_dependencies_inner(ast, snippets, snippet_params);
                output.push_str(&content);
              }
            }
            None => {
              // TODO Fehlermeldung? Müsste vorher bereits abgefangen sein.
            }
          }
        }
      }
      Rule::indented_reference => {
        let indentation = extract_indentation(&element);
        let identifier = extract_identifier(&element);
        let join_str = extract_join_str(&element)
          .replace("\\n", "\n");
        match snippet_params.get(identifier) {
          Some(param) => match param {
            ReferenceParam::Value(param) => output.push_str(&param),
            ReferenceParam::Reference(param) => {
              let ast = CodeblockParser::parse(Rule::codeblock, &param).expect("couldn't parse input.");
              let mut snippet_params = snippet_params.clone();
              for element in ast
                .clone()
                .filter(|element| match element.as_rule() {
                  Rule::reference => true,
                  Rule::indented_reference => true,
                  _ => false
                })
                .flat_map(|element| element.clone().into_inner())
                .filter(|element| {
                  match element.as_rule() {
                    Rule::attributes => true,
                    _ => false
                  }
                })
                .flat_map(|element| element.clone().into_inner())
                .filter(|element| match element.as_rule() {
                  Rule::attribute_param => true,
                  _ => false
                })
              {
                let key = element
                  .clone()
                  .into_inner()
                  .find(|element| match element.as_rule() {
                    Rule::identifier => true,
                    _ => false
                  })
                  .map(|element| element.as_str().to_string())
                  .unwrap_or("".to_string());
                let value = element
                  .into_inner()
                  .find(|element| match element.as_rule() {
                    Rule::value => true,
                    Rule::reference => true,
                    _ => false
                  })
                  .map(|element| match element.as_rule() {
                    Rule::value => ReferenceParam::Value(element.as_str().to_string()),
                    Rule::reference => ReferenceParam::Reference(element.as_str().to_string()),
                    _ => ReferenceParam::Value("".to_string())
                  })
                  .unwrap();
                snippet_params.insert(key, value);

              }
              let content = merge_dependencies_inner(ast, snippets, snippet_params);
              indent(&content, indentation, &mut output);
            }
          }
          None => match snippets.get(identifier) {
            Some(snippet) => {
              if snippet_params.is_empty() {
                let content = snippet.get_content(&join_str);
                indent(&content, indentation, &mut output);
              } else {
                let input = snippet.get_raw_content(&join_str);
                let ast = CodeblockParser::parse(Rule::codeblock, &input).expect("couldn't parse input.");
                let mut snippet_params = snippet_params.clone();
                for element in ast
                  .clone()
                  .filter(|element| match element.as_rule() {
                    Rule::reference => true,
                    Rule::indented_reference => true,
                    _ => false
                  })
                  .flat_map(|element| element.clone().into_inner())
                  .filter(|element| {
                    match element.as_rule() {
                      Rule::attributes => true,
                      _ => false
                    }
                  })
                  .flat_map(|element| element.clone().into_inner())
                  .filter(|element| match element.as_rule() {
                    Rule::attribute_param => true,
                    _ => false
                  })
                {
                  let key = element
                    .clone()
                    .into_inner()
                    .find(|element| match element.as_rule() {
                      Rule::identifier => true,
                      _ => false
                    })
                    .map(|element| element.as_str().to_string())
                    .unwrap_or("".to_string());
                  let value = element
                    .into_inner()
                    .find(|element| match element.as_rule() {
                      Rule::value => true,
                      Rule::reference => true,
                      _ => false
                    })
                    .map(|element| match element.as_rule() {
                      Rule::value => ReferenceParam::Value(element.as_str().to_string()),
                      Rule::reference => ReferenceParam::Reference(element.as_str().to_string()),
                      _ => ReferenceParam::Value("".to_string())
                    })
                    .unwrap();
                  snippet_params.insert(key, value);

                }
                let content = merge_dependencies_inner(ast, snippets, snippet_params);
                indent(&content, indentation, &mut output);
              }
            }
            None => {
              // TODO Fehlermeldung? Müsste vorher bereits abgefangen sein.
            }
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

fn indent(content: &str, indentation: &str, output: &mut String) -> () {
  for line in content.lines() {
    output.push_str("\n");
    output.push_str(indentation);
    output.push_str(line);
  }
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
  let ast = CodeblockParser::parse(Rule::codeblock, input).expect("couldn't parse input.");
  let mut snippet_params: HashMap<String, ReferenceParam> = HashMap::default();
  for element in ast
    .clone()
    .filter(|element| match element.as_rule() {
      Rule::reference => true,
      Rule::indented_reference => true,
      _ => false
    })
    .flat_map(|element| element.clone().into_inner())
    .filter(|element| {
      match element.as_rule() {
        Rule::attributes => true,
        _ => false
      }
    })
    .flat_map(|element| element.clone().into_inner())
    .filter(|element| match element.as_rule() {
      Rule::attribute_param => true,
      _ => false
    })
  {
    let key = element
      .clone()
      .into_inner()
      .find(|element| match element.as_rule() {
        Rule::identifier => true,
        _ => false
      })
      .map(|element| element.as_str().to_string())
      .unwrap_or("".to_string());
    let value = element
      .into_inner()
      .find(|element| match element.as_rule() {
        Rule::value => true,
        Rule::reference => true,
        _ => false
      })
      .map(|element| match element.as_rule() {
        Rule::value => ReferenceParam::Value(element.as_str().to_string()),
        Rule::reference => ReferenceParam::Reference(element.as_str().to_string()),
        _ => ReferenceParam::Value("".to_string())
      })
      .unwrap();
    snippet_params.insert(key, value);

  }

  merge_dependencies_inner(ast, snippets, snippet_params)
}
