use crate::{options, Element, ElementSpan, ListType, AST};
use std::io::{self, Write};

pub struct AsciidocWriter {}

impl AsciidocWriter {
    pub fn new() -> Self {
        AsciidocWriter {}
    }

    fn attribute_value_to_string(value: &crate::AttributeValue) -> String {
        match value {
            crate::AttributeValue::String(s) => s.clone(),
            crate::AttributeValue::Ref(r) => r.to_string(),
        }
    }

    fn write_element<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        match &element.element {
            Element::Title { level } => {
                self.write_title(element, *level, out)?;
            }
            Element::Paragraph => {
                self.write_paragraph(element, out)?;
            }
            Element::Text => {
                self.write_text(element, out)?;
            }
            Element::Styled => {
                self.write_styled(element, out)?;
            }
            Element::List(list_type) => {
                self.write_list(element, list_type, out)?;
            }
            Element::ListItem(level) => {
                self.write_list_item(element, *level, out)?;
            }
            Element::TypedBlock { kind } => {
                self.write_typed_block(element, kind, out)?;
            }
            Element::Table => {
                self.write_table(element, out)?;
            }
            Element::TableRow => {
                self.write_table_row(element, out)?;
            }
            Element::TableCell => {
                self.write_table_cell(element, out)?;
            }
            Element::Link => {
                self.write_link(element, out)?;
            }
            Element::XRef => {
                self.write_xref(element, out)?;
            }
            Element::Image => {
                self.write_image(element, out)?;
            }
            Element::Anchor => {
                self.write_anchor(element, out)?;
            }
            Element::Comment => {
                self.write_comment(element, out)?;
            }
            Element::Attribute(_) => {
                // Document attributes are handled separately
            }
            Element::IncludeElement(_) => {
                // TODO: Implement include element writing
                writeln!(out, "// TODO: IncludeElement")?;
            }
            Element::ExternalContent => {
                // TODO: Implement external content writing
                writeln!(out, "// TODO: ExternalContent")?;
            }
            Element::Error(msg) => {
                writeln!(out, "// ERROR: {}", msg)?;
            }
        }
        Ok(())
    }

    fn write_title<W: Write>(
        &mut self,
        element: &ElementSpan,
        level: u32,
        out: &mut W,
    ) -> crate::Result<()> {
        // AsciiDoc uses = for level 1, == for level 2, etc.
        let prefix = "=".repeat(level as usize);

        // The title text is stored in the "name" attribute
        let title = element.get_attribute("name").unwrap_or("");
        writeln!(out, "{} {}", prefix, title)?;

        Ok(())
    }

    fn write_paragraph<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        for child in &element.children {
            self.write_element(child, out)?;
        }
        writeln!(out)?;
        Ok(())
    }

    fn write_text<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        // Don't trim - spaces are significant in inline text
        write!(out, "{}", element.content)?;
        Ok(())
    }

    fn write_styled<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        // The styled content is stored in the "content" attribute
        let content = element.get_attribute("content").unwrap_or("");
        let style = element.get_attribute("style");

        match style {
            Some("strong") | Some("bold") => {
                write!(out, "*{}*", content)?;
            }
            Some("em") | Some("emphasis") | Some("italic") => {
                write!(out, "_{}_", content)?;
            }
            Some("monospaced") | Some("monospace") | Some("code") => {
                write!(out, "`{}`", content)?;
            }
            _ => {
                // Unknown style, just write content
                write!(out, "{}", content)?;
            }
        }

        Ok(())
    }

    fn write_list<W: Write>(
        &mut self,
        element: &ElementSpan,
        list_type: &ListType,
        out: &mut W,
    ) -> crate::Result<()> {
        // Process list items directly to maintain context about list type
        for child in &element.children {
            if let Element::ListItem(level) = &child.element {
                self.write_list_item_with_type(child, *level, list_type, out)?;
            }
        }
        Ok(())
    }

    fn write_list_item<W: Write>(
        &mut self,
        element: &ElementSpan,
        level: u32,
        out: &mut W,
    ) -> crate::Result<()> {
        // This shouldn't be called directly in list context, but keep for safety
        // Default to bullet list
        self.write_list_item_with_type(element, level, &ListType::Bullet, out)
    }

    fn write_list_item_with_type<W: Write>(
        &mut self,
        element: &ElementSpan,
        level: u32,
        list_type: &ListType,
        out: &mut W,
    ) -> crate::Result<()> {
        // Write the appropriate marker based on list type
        match list_type {
            ListType::Number => {
                let prefix = ".".repeat(level as usize);
                write!(out, "{} ", prefix)?;
            }
            ListType::Bullet => {
                let prefix = "*".repeat(level as usize);
                write!(out, "{} ", prefix)?;
            }
        }

        // Write list item content
        // Don't add extra newline - let children handle it
        for child in &element.children {
            match &child.element {
                Element::List(nested_type) => {
                    // Nested list - write on new line
                    writeln!(out)?;
                    self.write_list(child, nested_type, out)?;
                }
                Element::Paragraph => {
                    // For paragraphs in lists, write content inline (without the paragraph wrapper newline)
                    for grandchild in &child.children {
                        self.write_element(grandchild, out)?;
                    }
                    writeln!(out)?;
                }
                _ => {
                    self.write_element(child, out)?;
                }
            }
        }

        Ok(())
    }

    fn write_typed_block<W: Write>(
        &mut self,
        element: &ElementSpan,
        kind: &crate::BlockType,
        out: &mut W,
    ) -> crate::Result<()> {
        use crate::BlockType;

        // Write block attributes if present (exclude "content" attribute)
        let has_positional_attrs = !element.positional_attributes.is_empty();
        let named_attrs: Vec<_> = element
            .attributes
            .iter()
            .filter(|attr| attr.key != "content")
            .collect();

        if has_positional_attrs || !named_attrs.is_empty() {
            write!(out, "[")?;

            // Write positional attributes (like source, language)
            for (i, attr) in element.positional_attributes.iter().enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                match attr {
                    crate::AttributeValue::String(s) => write!(out, "{}", s)?,
                    crate::AttributeValue::Ref(r) => write!(out, "{}", r)?,
                }
            }

            // Write named attributes (except "content")
            for attr in named_attrs {
                let value = Self::attribute_value_to_string(&attr.value);
                write!(out, ",{}={}", attr.key, value)?;
            }

            writeln!(out, "]")?;
        }

        // Determine delimiter based on block type
        let delimiter = match kind {
            BlockType::Listing => "----",
            BlockType::Literal => "....",
            BlockType::Example => "====",
            BlockType::Sidebar => "****",
            BlockType::Quote => "____",
            BlockType::Passtrough => "++++",
            BlockType::Comment => "////",
        };

        writeln!(out, "{}", delimiter)?;

        // Write block content from the "content" attribute or element.content
        let content = element.get_attribute("content").unwrap_or(element.content);
        write!(out, "{}", content.trim())?;

        writeln!(out)?;
        writeln!(out, "{}", delimiter)?;

        Ok(())
    }

    fn write_table<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        // Write table attributes if present
        if !element.attributes.is_empty() {
            write!(out, "[")?;
            for (i, attr) in element.attributes.iter().enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                let value = Self::attribute_value_to_string(&attr.value);
                write!(out, "{}={}", attr.key, value)?;
            }
            writeln!(out, "]")?;
        }

        writeln!(out, "|===")?;

        for child in &element.children {
            self.write_element(child, out)?;
        }

        writeln!(out, "|===")?;
        Ok(())
    }

    fn write_table_row<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        for child in &element.children {
            self.write_element(child, out)?;
        }
        writeln!(out)?;
        Ok(())
    }

    fn write_table_cell<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        write!(out, "| ")?;

        for child in &element.children {
            self.write_element(child, out)?;
        }

        Ok(())
    }

    fn write_link<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        let url = element.get_attribute("url").unwrap_or("");

        write!(out, "link:{}[", url)?;

        // Link text is in positional_attributes[0]
        if let Some(link_text) = element.positional_attributes.get(0) {
            let text = Self::attribute_value_to_string(link_text);
            write!(out, "{}", text)?;
        }

        write!(out, "]")?;
        Ok(())
    }

    fn write_xref<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        let anchor = element.get_attribute("anchor").unwrap_or("");

        write!(out, "<<{}", anchor)?;

        if !element.children.is_empty() {
            write!(out, ",")?;
            for child in &element.children {
                self.write_element(child, out)?;
            }
        }

        write!(out, ">>")?;
        Ok(())
    }

    fn write_image<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        let path = element.get_attribute("path").unwrap_or("");

        write!(out, "image::{}[", path)?;

        // Write alt text from children or attributes
        if let Some(alt) = element.get_attribute("alt") {
            write!(out, "{}", alt)?;
        } else {
            for child in &element.children {
                self.write_element(child, out)?;
            }
        }

        write!(out, "]")?;
        Ok(())
    }

    fn write_anchor<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        if let Some(name) = element.get_attribute("name") {
            writeln!(out, "[[{}]]", name)?;
        }
        Ok(())
    }

    fn write_comment<W: Write>(
        &mut self,
        element: &ElementSpan,
        out: &mut W,
    ) -> crate::Result<()> {
        writeln!(out, "// {}", element.content.trim())?;
        Ok(())
    }
}

impl<W: io::Write> crate::Writer<W> for AsciidocWriter {
    fn write<'a>(&mut self, ast: AST, _args: &options::Opts, out: W) -> crate::Result<()> {
        let mut out = out;

        // Write document attributes first
        for attr in &ast.attributes {
            let value = Self::attribute_value_to_string(&attr.value);
            writeln!(out, ":{}: {}", attr.key, value)?;
        }

        if !ast.attributes.is_empty() {
            writeln!(out)?;
        }

        // Write all elements
        for (i, element) in ast.elements.iter().enumerate() {
            self.write_element(element, &mut out)?;

            // Add blank line between top-level elements
            // Don't add blank line after list items or within lists
            if i < ast.elements.len() - 1 {
                let next_element = &ast.elements[i + 1];
                match (&element.element, &next_element.element) {
                    // Don't add blank line between consecutive list items
                    (Element::ListItem(_), Element::ListItem(_)) => {}
                    // Don't add blank line after a list item
                    (Element::ListItem(_), _) => {}
                    // Don't add blank line before a list item
                    (_, Element::ListItem(_)) => {}
                    // Don't add blank line after a list
                    (Element::List(_), _) => {
                        writeln!(out)?;
                    }
                    // Add blank line between other elements
                    _ => {
                        writeln!(out)?;
                    }
                }
            }
        }

        Ok(())
    }
}
