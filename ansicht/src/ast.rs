#[derive(Debug, Clone, PartialEq)]
pub struct AST<'a> {
  pub content: &'a str,
  pub elements: Vec<ElementSpan>,
}

/// The basic element of a program
///
/// This is meant to form a tree of elements.
/// Every element holds references to its source, it
/// subelements and the attributes defined on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementSpan {
  // The source document. Could be empty if
  // e.g. it's the same as the source of it's
  // parent
  pub source: Option<String>,
  pub position: TextPosition,

  pub element: Element,
  /// The subelements of a nodes
  pub children: Vec<ElementSpan>,
  pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextPosition {
  BoundingBox {
    start: CursorPosition,
    end: CursorPosition,
  },
  Slice(Slice),
  // TODO Token Vector
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorPosition {
  pub line: usize,
  pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
  DataFlow(DataflowElement), // TypeFlow better name?
  Sequence(SequenceDiagramElement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataflowElement {
  /// A Dataset
  Dataset {
    id: usize,
  },
  /// Some kind of data processor
  /// Could be a function, a program etc
  /// The important thing is that it has data as inputs and outputs
  Function {
    id: usize,
  },
  Connection {
    from: usize,
    to: usize,
  },
  // Some element holding inner elements
  Container,
  // An element to decide which connection to take
  Decision,
  /// A wrongly formatted text or block
  Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SequenceDiagramElement {
  /// A message (or method or function call or event etc) from one
  /// participant to another
  Message {
    from: String,
    to: String,
    message: String,
    meta: Option<MetaData>,
  },
  /// A state that can be checked for the participants
  CheckedState {
    name: String,
    participants: Vec<String>,
  },
}

impl SequenceDiagramElement {
  pub fn message(from: &str, to: &str, message: &str) -> Self {
    Self::Message {
      from: from.to_string(),
      to: to.to_string(),
      message: message.to_string(),
      meta: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaData {}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
  Link(String),
  Position, // TODO Add information about the graphical position
}
