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

fn escape_text(input: &str) -> String {
  input.replace("<", "&lt;").replace(">", "&gt;")
}

// TODO Styles etc
fn write_html<T: io::Write>(input: &ElementSpan, out: &mut T) -> Result<()> {
  match &input.element {
    Element::Title { level } => {
      let title = input.get_attribute("name").unwrap_or("");

      out.write_all(&format!("<h{}", level).as_bytes())?;

      if level > &1 {
        let id = match input.get_attribute("anchor") {
          Some(id) => id.to_string(),
          None => "_".to_string() + &title.replace(" ", "_").to_lowercase(),
        };
        out.write_all(&format!(" id=\"{}\"", id).as_bytes())?;
      };

      out.write_all(&format!(">{}</h{}>\n", title, level).as_bytes())?;
    }
    Element::Paragraph => {
      out.write_all(b"<p>")?;
      for element in input.children.iter() {
        write_html(element, out)?;
      }
      out.write_all(b"</p>\n")?;
    }
    Element::Text => {
      out.write_all(input.content.as_bytes())?;
    }
    Element::Link => {
      let url = input.get_attribute("url").unwrap_or("");
      let content = match input.positional_attributes.get(0) {
        Some(value) => match value {
          AttributeValue::Ref(value) => value.to_string(),
          AttributeValue::String(value) => value.clone(),
        },
        None => "".to_string(),
      };

      out.write_all(&format!("<a href=\"{}\">{}</a>", url, content).as_bytes())?;
    }
    Element::Styled => {
      let style = input.get_attribute("style").unwrap_or("");
      let content = input.get_attribute("content").unwrap_or("");

      if style == "monospaced" {
        out.write_all(b"<code>")?;
      } else if style == "strong" {
        out.write_all(b"<strong>")?;
      }

      out.write_all(content.as_bytes())?;

      if style == "monospaced" {
        out.write_all(b"</code>")?;
      } else if style == "strong" {
        out.write_all(b"</strong>")?;
      };
    }
    Element::XRef => {
      let id = input.get_attribute("id").unwrap_or("");
      let content = input.get_attribute("content").unwrap_or(id.clone());

      out.write_all(&format!("<a href=\"#{}\">{}</a>", id, content).as_bytes())?;
    }
    Element::List => {
      let mut level = 1;

      out.write_all(b"<ul>\n")?;
      for element in input.children.iter() {
        if let Element::ListItem(item_level) = element.element {
          if item_level > level {
            out.write_all(b"<ul>\n")?;
            level = item_level;
          } else if item_level < level {
            out.write_all(b"</ul>\n")?;
            level = item_level;
          }
          write_html(element, out)?;
        }
      }
      out.write_all(b"</ul>\n")?;
    }
    Element::ListItem(_level) => {
      out.write_all(b"<li>\n")?;
      for element in input.children.iter() {
        write_html(element, out)?;
      }
      out.write_all(b"</li>\n")?;
    }
    Element::TypedBlock { kind } => {
      if kind == &BlockType::Comment {
        // Comments are not printed in html
        // TODO provide option to print comments
        return Ok(());
      }

      out.write_all(b"<div ")?;

      if let Some(id) = input.get_attribute("anchor") {
        out.write_all(&format!("id=\"{}\" ", id).as_bytes())?;
      };

      let class = match kind {
        BlockType::Listing => "listingblock",
        _ => "unknown-block",
      };
      out.write_all(&format!("class=\"{}\">\n", class).as_bytes())?;

      if let Some(title) = input.get_attribute("title") {
        out.write_all(&format!("<div class=\"title\">{}</div>\n", title).as_bytes())?;
      };

      if kind == &BlockType::Listing {
        out.write_all(b"<pre>")?;
      }

      let content = input.get_attribute("content").unwrap_or(input.content);
      out.write_all(escape_text(&content).as_bytes())?;

      if kind == &BlockType::Listing {
        out.write_all(b"</pre>\n")?;
      }

      out.write_all(b"</div>\n")?;
    }
    _ => {
      out.write_all(
        &format!(
          "<NOT-YET-SUPPORTED:{:?}>{}</NOT-YET-SUPPORTED>\n",
          input.element, input.content
        )
        .as_bytes(),
      )?;
    }
  }

  Ok(())
}
