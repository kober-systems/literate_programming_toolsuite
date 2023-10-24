use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AST<'a> {
  pub content: &'a str,
  pub elements: Vec<ElementSpan<'a>>,
  pub attributes: Vec<Attribute<'a>>,
}

impl AST<'_> {
  pub fn get_attribute(&self, name: &str) -> Option<&str> {
    for attribute in self.attributes.iter() {
      if &attribute.key == name {
        return match &attribute.value {
          AttributeValue::Ref(value) => Some(value),
          AttributeValue::String(value) => Some(value.as_str()),
        };
      }
    }

    None
  }
}

/// The basic element of a document
///
/// This is meant to form a tree of document element.
/// Every element holds references to its source, it
/// subelements and the attributes defined on it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ElementSpan<'a> {
  // The source document. Could be empty if
  // e.g. it's the same as the source of it's
  // parent
  pub source: Option<String>,
  // A string reference to the source
  pub content: &'a str,
  // TODO Add start and end point
  pub start: usize,
  pub end: usize,
  /// We count the lines for usage in other tools
  pub start_line: usize,
  pub start_col: usize,
  pub end_line: usize,
  pub end_col: usize,

  pub element: Element<'a>,
  /// The subelements of a nodes
  pub children: Vec<ElementSpan<'a>>,
  /// The attributes applying to that node and
  /// all children
  pub positional_attributes: Vec<AttributeValue<'a>>,
  /// The attributes applying to that node and
  /// all children
  pub attributes: Vec<Attribute<'a>>,
}

impl ElementSpan<'_> {
  pub fn get_attribute(&self, name: &str) -> Option<&str> {
    for attribute in self.attributes.iter() {
      if &attribute.key == name {
        return match &attribute.value {
          AttributeValue::Ref(value) => Some(value),
          AttributeValue::String(value) => Some(value.as_str()),
        };
      }
    }

    None
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Element<'a> {
  Attribute(#[serde(borrow)] Attribute<'a>),
  /// A section of ignored text
  Comment,
  /// A text paragraph
  Paragraph,
  /// A header section
  Title {
    level: u32,
  },
  Table,
  List(ListType),
  Image,
  Anchor,
  /// Holds all blocks with special content and the type
  /// TODO Could be done with ExternalContent and all known
  /// Types here direktly
  TypedBlock {
    kind: BlockType,
  },
  /// Holds content which is not przessed direktly by
  /// asciidoctrine. It can be anything. Outputs or
  /// postprocessors could use or ignore it at their
  /// will (e.g. videos)
  ExternalContent,
  /// Holds a reference to the include statement
  /// and a document inside
  IncludeElement(IncludeElement<'a>),

  /// The following variants are inline elements nested
  /// inside a conainer element

  /// Element with a special style. The attributes define the kind of style
  Styled,
  /// A chunk of text.
  Text,
  /// An internal reference or link
  XRef,
  /// An external link
  Link,
  /// A list item
  ListItem(u32),
  /// A table row
  TableRow,
  /// A table cell
  TableCell,
  /// A wrong formatted text or block
  Error(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ListType {
  Bullet,
  Number,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BlockType {
  Comment,
  Passtrough,
  Listing,
  Literal,
  Sidebar,
  Quote,
  Example,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AttributeValue<'a> {
  String(String),
  Ref(&'a str),
}

impl AttributeValue<'_> {
  pub fn as_str(&self) -> &str {
    match self {
      AttributeValue::Ref(value) => value,
      AttributeValue::String(value) => value.as_str(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Attribute<'a> {
  pub key: String,
  #[serde(borrow)]
  pub value: AttributeValue<'a>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IncludeElement<'a> {
  #[serde(borrow)]
  pub inner: AST<'a>,
}
