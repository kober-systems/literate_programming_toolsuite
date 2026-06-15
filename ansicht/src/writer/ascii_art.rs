use std::io::Write;

use crate::{Element, SequenceDiagramElement, Writer, AST};

pub struct AsciiArtWriter {
  pub indent: usize,
}

impl Default for AsciiArtWriter {
  fn default() -> Self {
    Self { indent: 5 }
  }
}

impl AsciiArtWriter {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<T: Write> Writer<T> for AsciiArtWriter {
  fn write<'a>(&mut self, ast: AST<'a>, mut out: T) -> crate::Result<()> {
    let model = SequenceRenderModel::from_ast(&ast);

    // Calculate layout
    let layout = LayoutCalculator::new(&model.participants, &model.messages, self.indent);

    // Draw header
    draw_participant_boxes(
      &mut out,
      &layout,
      &model.participants,
      BoxStyle::SimpleBox,
      false,
      true,
    )?;

    // Draw sequence events
    let event_count = model.events.len();
    for (index, event) in model.events.into_iter().enumerate() {
      match event {
        SequenceEvent::CheckedState(name) => {
          draw_checked_state_box(&mut out, &layout, &name)?;
        }
        SequenceEvent::Message(message) => {
          draw_message_text(&mut out, &layout, &model.participants, &message)?;
          draw_message_arrow(&mut out, &layout, &model.participants, &message)?;
        }
      }
      if index + 1 < event_count {
        draw_empty_line(&mut out, &layout, &model.participants)?;
      }
    }

    // Draw footer
    draw_participant_boxes(
      &mut out,
      &layout,
      &model.participants,
      BoxStyle::SimpleBox,
      true,
      false,
    )?;

    Ok(())
  }
}

enum SequenceEvent {
  CheckedState(String),
  Message(MessageEvent),
}

#[derive(Clone)]
struct MessageEvent {
  from: String,
  to: String,
  message: String,
}

struct SequenceRenderModel {
  participants: Vec<String>,
  messages: Vec<MessageEvent>,
  events: Vec<SequenceEvent>,
}

impl SequenceRenderModel {
  fn from_ast(ast: &AST<'_>) -> Self {
    let mut participants = Vec::new();
    let mut messages = Vec::new();
    let mut events = Vec::new();

    for element_span in &ast.elements {
      match &element_span.element {
        Element::Sequence(SequenceDiagramElement::CheckedState {
          name,
          participants: ps,
        }) => {
          for participant in ps {
            push_unique(&mut participants, participant.clone());
          }
          events.push(SequenceEvent::CheckedState(name.clone()));
        }
        Element::Sequence(SequenceDiagramElement::Message {
          from, to, message, ..
        }) => {
          push_unique(&mut participants, from.clone());
          push_unique(&mut participants, to.clone());
          let message_event = MessageEvent {
            from: from.clone(),
            to: to.clone(),
            message: message.clone(),
          };
          messages.push(message_event.clone());
          events.push(SequenceEvent::Message(message_event));
        }
        _ => {}
      }
    }

    Self {
      participants,
      messages,
      events,
    }
  }
}

fn push_unique(items: &mut Vec<String>, item: String) {
  if !items.contains(&item) {
    items.push(item);
  }
}

#[derive(Debug, Clone, Copy)]
struct BoxArea {
  left: usize,
  right: usize,
}

impl BoxArea {
  fn new(left: usize, right: usize) -> Self {
    Self { left, right }
  }

  fn from_left_and_width(left: usize, width: usize) -> Self {
    Self {
      left,
      right: left + width + 1,
    }
  }

  fn width(self) -> usize {
    self.right - self.left - 1
  }

  fn marker_position(self) -> usize {
    self.left + self.width() / 2
  }
}

struct MessageSpan {
  left: usize,
  right: usize,
  going_right: bool,
}

struct LayoutCalculator {
  participant_widths: Vec<usize>,
  participant_lefts: Vec<usize>,
  participant_positions: Vec<usize>,
  line_width: usize,
}

impl LayoutCalculator {
  fn new(participants: &[String], messages: &[MessageEvent], indent: usize) -> Self {
    let participant_widths = participants.iter().map(|p| p.len()).collect::<Vec<_>>();
    let lifeline_gaps = calculate_lifeline_gaps(participants, messages);

    let mut participant_lefts = Vec::new();
    let mut participant_positions = Vec::new();
    let mut lifeline_pos = indent + participant_widths.first().copied().unwrap_or(0) / 2;

    for (index, width) in participant_widths.iter().enumerate() {
      participant_positions.push(lifeline_pos);
      participant_lefts.push(lifeline_pos - width / 2);

      if let Some(gap) = lifeline_gaps.get(index) {
        lifeline_pos += gap;
      }
    }

    let line_width = participant_lefts
      .last()
      .zip(participant_widths.last())
      .map(|(left, width)| left + width + 2 + indent)
      .unwrap_or(indent);

    LayoutCalculator {
      participant_widths,
      participant_lefts,
      participant_positions,
      line_width,
    }
  }

  fn participant_width(&self, index: usize) -> usize {
    self.participant_widths[index]
  }

  fn participant_left(&self, index: usize) -> usize {
    self.participant_lefts[index]
  }

  fn participant_right(&self, index: usize) -> usize {
    self.participant_box(index).right
  }

  fn participant_box(&self, index: usize) -> BoxArea {
    BoxArea::from_left_and_width(self.participant_left(index), self.participant_width(index))
  }

  fn checked_state_box(&self, label: &str) -> Option<BoxArea> {
    if self.participant_positions.is_empty() {
      return None;
    }

    let left = self.participant_left(0);
    let last_participant = self.participant_positions.len() - 1;
    let right = self
      .participant_right(last_participant)
      .max(left + label.len() + 3);

    Some(BoxArea::new(left, right))
  }

  fn get_participant_pos(&self, name: &str, participants: &[String]) -> Option<usize> {
    participants
      .iter()
      .position(|p| p == name)
      .and_then(|i| self.participant_positions.get(i).copied())
  }

  fn message_span(&self, participants: &[String], from: &str, to: &str) -> MessageSpan {
    let from_pos = self.get_participant_pos(from, participants).unwrap_or(0);
    let to_pos = self.get_participant_pos(to, participants).unwrap_or(0);

    if from_pos < to_pos {
      MessageSpan {
        left: from_pos,
        right: to_pos,
        going_right: true,
      }
    } else {
      MessageSpan {
        left: to_pos,
        right: from_pos,
        going_right: false,
      }
    }
  }
}

fn calculate_lifeline_gaps(participants: &[String], messages: &[MessageEvent]) -> Vec<usize> {
  let mut gaps = Vec::new();

  for index in 0..participants.len().saturating_sub(1) {
    let left = &participants[index];
    let right = &participants[index + 1];
    let longest_adjacent_message = messages
      .iter()
      .filter(|message| {
        (message.from.as_str() == left.as_str() && message.to.as_str() == right.as_str())
          || (message.from.as_str() == right.as_str() && message.to.as_str() == left.as_str())
      })
      .map(|message| message.message.len())
      .max()
      .unwrap_or(0);

    let minimum_gap_for_boxes =
      participants[index].len() / 2 + participants[index + 1].len() / 2 + 5;
    let minimum_gap_for_message = longest_adjacent_message + 1;

    gaps.push(minimum_gap_for_boxes.max(minimum_gap_for_message));
  }

  gaps
}

struct Canvas {
  cells: Vec<char>,
}

impl Canvas {
  fn new(width: usize) -> Self {
    Self {
      cells: vec![' '; width],
    }
  }

  fn put(&mut self, x: usize, ch: char) {
    if x < self.cells.len() {
      self.cells[x] = ch;
    }
  }

  fn text(&mut self, x: usize, text: &str, end: usize) {
    for (offset, ch) in text.chars().enumerate() {
      let pos = x + offset;
      if pos < end && pos < self.cells.len() {
        self.cells[pos] = ch;
      }
    }
  }

  fn hline(&mut self, from: usize, to: usize, ch: char) {
    for pos in from..to {
      self.put(pos, ch);
    }
  }

  fn lifelines(&mut self, layout: &LayoutCalculator) {
    for pos in &layout.participant_positions {
      self.put(*pos, '│');
    }
  }

  fn write_to(&self, out: &mut dyn Write) -> crate::Result<()> {
    let text = self.cells.iter().collect::<String>().trim_end().to_string();
    writeln!(out, "{}", text)
      .map_err(|e| crate::Error::ParseError(format!("Failed to write: {}", e)))
  }
}

#[derive(Debug, Clone, Copy)]
enum BoxStyle {
  SimpleBox,
  DoubleBorderedBox,
}

impl BoxStyle {
  fn vertical(self) -> char {
    match self {
      BoxStyle::SimpleBox => '│',
      BoxStyle::DoubleBorderedBox => '║',
    }
  }

  fn horizontal(self) -> char {
    match self {
      BoxStyle::SimpleBox => '─',
      BoxStyle::DoubleBorderedBox => '═',
    }
  }

  fn top_left(self) -> char {
    match self {
      BoxStyle::SimpleBox => '┌',
      BoxStyle::DoubleBorderedBox => '╔',
    }
  }

  fn top_right(self) -> char {
    match self {
      BoxStyle::SimpleBox => '┐',
      BoxStyle::DoubleBorderedBox => '╗',
    }
  }

  fn bottom_left(self) -> char {
    match self {
      BoxStyle::SimpleBox => '└',
      BoxStyle::DoubleBorderedBox => '╚',
    }
  }

  fn bottom_right(self) -> char {
    match self {
      BoxStyle::SimpleBox => '┘',
      BoxStyle::DoubleBorderedBox => '╝',
    }
  }

  fn top_lifeline_marker(self) -> char {
    match self {
      BoxStyle::SimpleBox => '┴',
      BoxStyle::DoubleBorderedBox => '╧',
    }
  }

  fn bottom_lifeline_marker(self) -> char {
    match self {
      BoxStyle::SimpleBox => '┬',
      BoxStyle::DoubleBorderedBox => '╤',
    }
  }
}

enum BoxRow<'a> {
  Top { lifeline_marker: Option<char> },
  Label(&'a str),
  Bottom { lifeline_marker: Option<char> },
}

fn draw_box_row(canvas: &mut Canvas, area: BoxArea, row: BoxRow<'_>, style: BoxStyle) {
  match row {
    BoxRow::Label(label) => {
      canvas.put(area.left, style.vertical());
      canvas.text(area.left + 1, label, area.right);
      canvas.put(area.right, style.vertical());
    }
    BoxRow::Top { lifeline_marker } => {
      canvas.put(area.left, style.top_left());
      canvas.hline(area.left + 1, area.right, style.horizontal());
      if let Some(marker) = lifeline_marker {
        canvas.put(area.marker_position(), marker);
      }
      canvas.put(area.right, style.top_right());
    }
    BoxRow::Bottom { lifeline_marker } => {
      canvas.put(area.left, style.bottom_left());
      canvas.hline(area.left + 1, area.right, style.horizontal());
      if let Some(marker) = lifeline_marker {
        canvas.put(area.marker_position(), marker);
      }
      canvas.put(area.right, style.bottom_right());
    }
  }
}

fn draw_participant_boxes(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
  style: BoxStyle,
  top_lifeline_marker: bool,
  bottom_lifeline_marker: bool,
) -> crate::Result<()> {
  let mut canvas = Canvas::new(layout.line_width);
  for (i, _) in participants.iter().enumerate() {
    draw_box_row(
      &mut canvas,
      layout.participant_box(i),
      BoxRow::Top {
        lifeline_marker: if top_lifeline_marker {
          Some(style.top_lifeline_marker())
        } else {
          None
        },
      },
      style,
    );
  }
  canvas.write_to(out)?;

  let mut canvas = Canvas::new(layout.line_width);
  for (i, participant) in participants.iter().enumerate() {
    draw_box_row(
      &mut canvas,
      layout.participant_box(i),
      BoxRow::Label(participant),
      style,
    );
  }
  canvas.write_to(out)?;

  let mut canvas = Canvas::new(layout.line_width);
  for (i, _) in participants.iter().enumerate() {
    draw_box_row(
      &mut canvas,
      layout.participant_box(i),
      BoxRow::Bottom {
        lifeline_marker: if bottom_lifeline_marker {
          Some(style.bottom_lifeline_marker())
        } else {
          None
        },
      },
      style,
    );
  }
  canvas.write_to(out)?;

  Ok(())
}

fn draw_checked_state_box(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  name: &str,
) -> crate::Result<()> {
  let style = BoxStyle::DoubleBorderedBox;
  let Some(area) = layout.checked_state_box(name) else {
    return Ok(());
  };

  let mut canvas = Canvas::new(layout.line_width.max(area.right + 1));
  draw_box_row(
    &mut canvas,
    area,
    BoxRow::Top {
      lifeline_marker: None,
    },
    style,
  );
  for lifeline in &layout.participant_positions {
    canvas.put(*lifeline, style.top_lifeline_marker());
  }
  canvas.write_to(out)?;

  let mut canvas = Canvas::new(layout.line_width.max(area.right + 1));
  draw_box_row(&mut canvas, area, BoxRow::Label(name), style);
  canvas.write_to(out)?;

  let mut canvas = Canvas::new(layout.line_width.max(area.right + 1));
  draw_box_row(
    &mut canvas,
    area,
    BoxRow::Bottom {
      lifeline_marker: None,
    },
    style,
  );
  canvas.put(area.left, style.bottom_left());
  canvas.put(area.right, style.bottom_right());
  for lifeline in &layout.participant_positions {
    canvas.put(*lifeline, style.bottom_lifeline_marker());
  }
  canvas.write_to(out)?;

  Ok(())
}

fn draw_message_text(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
  message: &MessageEvent,
) -> crate::Result<()> {
  let mut canvas = Canvas::new(layout.line_width);

  let span = layout.message_span(participants, &message.from, &message.to);

  canvas.lifelines(layout);

  // Draw message text in the middle
  let text_start = span.left + 1;
  let text_end = span.right;
  if text_start < text_end {
    let available = text_end - text_start;
    let trimmed = if message.message.len() > available {
      &message.message[..available.min(message.message.len())]
    } else {
      &message.message
    };
    let msg_start = text_start + (available.saturating_sub(trimmed.len()) + 1) / 2;
    canvas.text(msg_start, trimmed, span.right);
  }

  canvas.write_to(out)?;

  Ok(())
}

fn draw_message_arrow(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
  message: &MessageEvent,
) -> crate::Result<()> {
  let mut canvas = Canvas::new(layout.line_width);

  let span = layout.message_span(participants, &message.from, &message.to);

  canvas.lifelines(layout);

  // Draw arrow
  if span.going_right {
    canvas.hline(span.left + 1, span.right, '─');
    if span.right > 0 {
      canvas.put(span.right - 1, '>');
    }
  } else {
    canvas.put(span.left + 1, '<');
    for pos in span.left + 2..span.right {
      if (pos - (span.left + 2)) % 2 == 0 {
        canvas.put(pos, '─');
      }
    }
  }

  canvas.write_to(out)?;

  Ok(())
}

fn draw_empty_line(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  _participants: &[String],
) -> crate::Result<()> {
  let mut canvas = Canvas::new(layout.line_width);
  canvas.lifelines(layout);
  canvas.write_to(out)?;
  Ok(())
}
