pub use crate::ast::*;
use crate::options::Opts;
use crate::util::Env;
use crate::Result;
use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel, CodeBlockKind, Options};

pub struct MarkdownReader {}

impl MarkdownReader {
  pub fn new() -> Self {
    MarkdownReader {}
  }

  fn byte_offset_to_position(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in content.char_indices() {
      if i >= offset {
        break;
      }
      if ch == '\n' {
        line += 1;
        col = 1;
      } else {
        col += 1;
      }
    }

    (line, col)
  }

  fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
      HeadingLevel::H1 => 1,
      HeadingLevel::H2 => 2,
      HeadingLevel::H3 => 3,
      HeadingLevel::H4 => 4,
      HeadingLevel::H5 => 5,
      HeadingLevel::H6 => 6,
    }
  }

  fn span<'a>(input: &'a str, start: usize, end: usize, element: Element<'a>, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    let (start_line, start_col) = Self::byte_offset_to_position(input, start);
    let (end_line, end_col) = Self::byte_offset_to_position(input, end);
    ElementSpan {
      source: None, content: &input[start..end], element, start, end,
      start_line, start_col, end_line, end_col, children,
      positional_attributes: Vec::new(), attributes: Vec::new(),
    }
  }

  // A deeper quote marker can be treated as a lazy paragraph continuation by
  // the Markdown parser. Recover the explicit quote structure for plain lines.
  fn nested_quote<'a>(input: &'a str, start: usize, end: usize) -> ElementSpan<'a> {
    let mut children = Vec::new();
    let mut cursor = start;
    while cursor < end {
      let line_end = input[cursor..end].find('\n').map_or(end, |n| cursor + n + 1);
      let prefix = if input[cursor..line_end].starts_with("> ") { 2 } else { 1 };
      if input[cursor + prefix..line_end].starts_with("> ") {
        let nested_start = cursor + prefix;
        let mut group_end = line_end;
        while group_end < end && input[group_end..end].starts_with("> > ") {
          group_end += input[group_end..end].find('\n').map_or(end - group_end, |n| n + 1);
        }
        children.push(Self::nested_quote(input, nested_start, group_end));
        cursor = group_end;
      } else {
        let text_start = cursor + prefix;
        let text_end = if input.as_bytes()[line_end - 1] == b'\n' { line_end - 1 } else { line_end };
        children.push(Self::span(input, text_start, line_end, Element::Paragraph,
          vec![Self::span(input, text_start, text_end, Element::Text, vec![])]));
        cursor = line_end;
      }
    }
    Self::span(input, start, end, Element::TypedBlock { kind: BlockType::Quote }, children)
  }

  fn convert_events<'a>(&self, input: &'a str) -> Vec<ElementSpan<'a>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(input, options);

    let mut elements = Vec::new();
    let mut stack: Vec<ElementSpan<'a>> = Vec::new();
    let mut current_text = String::new();

    for (event, range) in parser.into_offset_iter() {
      match event {
        Event::Start(tag) => {
          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let element = match tag {
            Tag::Paragraph => Element::Paragraph,

            Tag::Heading { level, .. } => Element::Title {
              level: Self::heading_level_to_u32(level),
            },

            Tag::BlockQuote(_) => Element::TypedBlock {
              kind: BlockType::Quote,
            },

            Tag::CodeBlock(kind) => {
              let lang = match kind {
                CodeBlockKind::Fenced(lang) => {
                  if lang.is_empty() {
                    None
                  } else {
                    Some(lang.to_string())
                  }
                }
                CodeBlockKind::Indented => None,
              };

              let mut elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::TypedBlock {
                  kind: BlockType::Listing,
                },
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: Vec::new(),
              };

              if let Some(lang) = lang {
                elem_span
                  .positional_attributes
                  .push(AttributeValue::String("source".to_string()));
                elem_span
                  .positional_attributes
                  .push(AttributeValue::String(lang));
              }

              stack.push(elem_span);
              continue;
            }

            Tag::List(start_number) => {
              if start_number.is_some() {
                Element::List(ListType::Number)
              } else {
                Element::List(ListType::Bullet)
              }
            }

            Tag::Item => Element::ListItem(1),

            Tag::Table(_) => Element::Table,

            Tag::TableHead => {
              // We'll handle this as a special table row
              Element::TableRow
            }

            Tag::TableRow => Element::TableRow,

            Tag::TableCell => Element::TableCell,

            Tag::Emphasis => {
              let elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::Styled,
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: vec![Attribute {
                  key: "style".to_string(),
                  value: AttributeValue::Ref("em"),
                }],
              };
              stack.push(elem_span);
              continue;
            }

            Tag::Strong => {
              let elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::Styled,
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: vec![Attribute {
                  key: "style".to_string(),
                  value: AttributeValue::Ref("strong"),
                }],
              };
              stack.push(elem_span);
              continue;
            }

            Tag::Strikethrough => {
              let elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::Styled,
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: vec![Attribute {
                  key: "style".to_string(),
                  value: AttributeValue::Ref("strikethrough"),
                }],
              };
              stack.push(elem_span);
              continue;
            }

            Tag::Link {
              dest_url, title, ..
            } => {
              let dest_str = dest_url.to_string();
              let title_str = title.to_string();

              let mut elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::Link,
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: vec![Attribute {
                  key: "url".to_string(),
                  value: AttributeValue::String(dest_str.clone()),
                }],
              };

              if !title_str.is_empty() {
                elem_span.attributes.push(Attribute {
                  key: "title".to_string(),
                  value: AttributeValue::String(title_str),
                });
              }

              // Determine protocol
              if let Some(proto_end) = dest_str.find("://") {
                let protocol = &dest_str[..proto_end];
                elem_span.attributes.push(Attribute {
                  key: "protocol".to_string(),
                  value: AttributeValue::String(protocol.to_string()),
                });
              }

              stack.push(elem_span);
              continue;
            }

            Tag::Image {
              dest_url, title, ..
            } => {
              let dest_str = dest_url.to_string();
              let title_str = title.to_string();

              let mut elem_span = ElementSpan {
                source: None,
                content: &input[range.clone()],
                element: Element::Image,
                start: range.start,
                end: range.end,
                start_line,
                start_col,
                end_line,
                end_col,
                children: Vec::new(),
                positional_attributes: Vec::new(),
                attributes: vec![Attribute {
                  key: "path".to_string(),
                  value: AttributeValue::String(dest_str),
                }],
              };

              if !title_str.is_empty() {
                elem_span
                  .positional_attributes
                  .push(AttributeValue::String(title_str));
              }

              stack.push(elem_span);
              continue;
            }

            Tag::HtmlBlock => {
              // HTML blocks will have their content collected via Event::Html
              Element::TypedBlock {
                kind: BlockType::Passtrough,
              }
            }

            _ => {
              // For unsupported tags, create a generic element
              Element::Text
            }
          };

          let elem_span = ElementSpan {
            source: None,
            content: &input[range.clone()],
            element,
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: Vec::new(),
          };

          stack.push(elem_span);
        }

        Event::End(tag) => {
          match tag {
            TagEnd::HtmlBlock => {
              if let Some(mut elem) = stack.pop() {
                let mut content = Self::span(input, elem.start, elem.end,
                  Element::TypedBlock { kind: BlockType::Passtrough }, vec![]);
                content.attributes.push(Attribute {
                  key: "content".to_string(),
                  value: AttributeValue::String(content.content.to_string()),
                });
                elem.children = vec![content];
                if let Some(parent) = stack.last_mut() {
                  parent.children.push(elem);
                } else {
                  elements.push(elem);
                }
              }
            }
            TagEnd::CodeBlock => {
              // For code blocks, the content is in current_text
              if let Some(mut elem) = stack.pop() {
                elem.attributes.push(Attribute {
                  key: "content".to_string(),
                  value: AttributeValue::String(current_text.clone()),
                });
                current_text.clear();

                if let Some(parent) = stack.last_mut() {
                  parent.children.push(elem);
                } else {
                  elements.push(elem);
                }
              }
            }
            _ => {
              if let Some(elem) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                  parent.children.push(elem);
                } else {
                  elements.push(elem);
                }
              }
            }
          }
        }

        Event::Text(text) => {
          // Check if we're inside a code block
          if let Some(elem) = stack.last() {
            if let Element::TypedBlock {
              kind: BlockType::Listing,
            } = elem.element
            {
              current_text.push_str(&text);
              continue;
            }
          }

          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let text_elem = ElementSpan {
            source: None,
            content: &input[range.clone()],
            element: Element::Text,
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: Vec::new(),
          };

          if let Some(parent) = stack.last_mut() {
            parent.children.push(text_elem);
          } else {
            elements.push(text_elem);
          }
        }

        Event::Code(code) => {
          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let code_elem = ElementSpan {
            source: None,
            content: &input[range.clone()],
            element: Element::Styled,
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: vec![
              Attribute {
                key: "style".to_string(),
                value: AttributeValue::Ref("monospaced"),
              },
              Attribute {
                key: "content".to_string(),
                value: AttributeValue::String(code.to_string()),
              },
            ],
          };

          if let Some(parent) = stack.last_mut() {
            parent.children.push(code_elem);
          } else {
            elements.push(code_elem);
          }
        }

        Event::Html(html) | Event::InlineHtml(html) => {
          // Treat HTML as passthrough content
          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let html_elem = ElementSpan {
            source: None,
            content: &input[range.clone()],
            element: Element::TypedBlock {
              kind: BlockType::Passtrough,
            },
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: vec![Attribute {
              key: "content".to_string(),
              value: AttributeValue::String(html.to_string()),
            }],
          };

          if let Some(parent) = stack.last_mut() {
            parent.children.push(html_elem);
          } else {
            elements.push(html_elem);
          }
        }

        Event::SoftBreak | Event::HardBreak => {
          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let break_elem = ElementSpan {
            source: None,
            content: if matches!(event, Event::SoftBreak) {
              "\n"
            } else {
              "\n"
            },
            element: Element::Text,
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: Vec::new(),
          };

          if let Some(parent) = stack.last_mut() {
            parent.children.push(break_elem);
          }
        }

        Event::Rule => {
          let (start_line, start_col) = Self::byte_offset_to_position(input, range.start);
          let (end_line, end_col) = Self::byte_offset_to_position(input, range.end);

          let rule_elem = ElementSpan {
            source: None,
            content: &input[range.clone()],
            element: Element::ExternalContent,
            start: range.start,
            end: range.end,
            start_line,
            start_col,
            end_line,
            end_col,
            children: Vec::new(),
            positional_attributes: Vec::new(),
            attributes: vec![Attribute {
              key: "type".to_string(),
              value: AttributeValue::Ref("horizontal-rule"),
            }],
          };

          elements.push(rule_elem);
        }

        Event::TaskListMarker(checked) => {
          // Add checked attribute to the current list item
          if let Some(parent) = stack.last_mut() {
            if matches!(parent.element, Element::ListItem(_)) {
              parent.attributes.push(Attribute {
                key: "checked".to_string(),
                value: AttributeValue::String(checked.to_string()),
              });
            }
          }
        }

        _ => {}
      }
    }

    for elem in &mut elements {
      if matches!(elem.element, Element::TypedBlock { kind: BlockType::Quote })
        && elem.content.contains("\n> > ")
      {
        *elem = Self::nested_quote(input, elem.start, elem.end);
      }
    }
    elements
  }
}

impl crate::Reader for MarkdownReader {
  fn parse<'a>(&self, input: &'a str, args: &Opts, _env: &mut Env) -> Result<AST<'a>> {
    let mut attributes = Vec::new();

    if let Some(path) = &args.input {
      if let Some(path) = path.to_str() {
        attributes.push(Attribute {
          key: "source".to_string(),
          value: AttributeValue::String(path.to_string()),
        });
      }
    }

    Ok(AST {
      content: input,
      elements: self.convert_events(input),
      attributes,
    })
  }
}
