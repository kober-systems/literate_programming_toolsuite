pub use crate::ast::*;
use crate::Result;
use std::io;

pub struct HtmlWriter {}

impl HtmlWriter {
  pub fn new() -> Self {
    HtmlWriter {}
  }
}

impl<T: io::Write> crate::Writer<T> for HtmlWriter {
  fn write<'a>(&self, ast: AST, mut out: T) -> Result<()> {
    for element in ast.elements.iter() {
      write_html(element, &mut out)?;
    }
    out.flush()?;

    Ok(())
  }
}

// TODO Styles etc
fn write_html<T: io::Write>(input: &ElementSpan, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Title { level } => {
      out.write_all(
        &format!(
          "<h{}>{}</h{}>\n",
          level,
          input.get_attribute("name").unwrap_or(""),
          level
        )
        .as_bytes(),
      )?;
    }
    _ => {
      out.write_all(
        &format!("<NOT-YET-SUPPORTED>{}</NOT-YET-SUPPORTED>\n", input.content).as_bytes(),
      )?;
    }
  }

  Ok(())
}
