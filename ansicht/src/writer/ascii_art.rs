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
    // Extract participants and messages
    let mut participants = Vec::new();
    let mut messages = Vec::new();

    for element_span in &ast.elements {
      match &element_span.element {
        Element::Sequence(SequenceDiagramElement::CheckedState { participants: ps, .. }) => {
          participants.extend(ps.clone());
        }
        Element::Sequence(SequenceDiagramElement::Message { from, to, message, .. }) => {
          if !participants.contains(from) {
            participants.push(from.clone());
          }
          if !participants.contains(to) {
            participants.push(to.clone());
          }
          messages.push((from.clone(), to.clone(), message.clone()));
        }
        _ => {}
      }
    }

    if participants.is_empty() || messages.is_empty() {
      return Ok(());
    }

    // Calculate layout
    let layout = LayoutCalculator::new(&participants, self.indent);

    // Draw top boxes
    draw_header(&mut out, &layout, &participants)?;

    // Draw messages
    let message_count = messages.len();
    for (index, (from, to, message)) in messages.into_iter().enumerate() {
      draw_message_text(&mut out, &layout, &participants, &from, &to, &message)?;
      draw_message_arrow(&mut out, &layout, &participants, &from, &to)?;
      if index + 1 < message_count {
        draw_empty_line(&mut out, &layout, &participants)?;
      }
    }

    // Draw bottom boxes
    draw_footer(&mut out, &layout, &participants)?;

    Ok(())
  }
}

struct LayoutCalculator {
  participant_width: usize,
  participant_positions: Vec<usize>,
  line_width: usize,
}

impl LayoutCalculator {
  fn new(participants: &[String], indent: usize) -> Self {
    let participant_width = participants.iter().map(|p| p.len()).max().unwrap_or(6).max(3);
    let box_width = participant_width + 2; // +2 for borders
    let spacing = 16;

    let mut participant_positions = Vec::new();
    let mut pos = indent;
    for _ in participants {
      participant_positions.push(pos + participant_width / 2); // Lifeline position inside the box
      pos += box_width + spacing;
    }

    let line_width = pos.saturating_sub(spacing) + indent;

    LayoutCalculator {
      participant_width,
      participant_positions,
      line_width,
    }
  }

  fn get_participant_pos(&self, name: &str, participants: &[String]) -> Option<usize> {
    participants
      .iter()
      .position(|p| p == name)
      .and_then(|i| self.participant_positions.get(i).copied())
  }
}

fn write_line(out: &mut dyn Write, line: &[char]) -> crate::Result<()> {
  let text = line.iter().collect::<String>().trim_end().to_string();
  writeln!(out, "{}", text)
    .map_err(|e| crate::Error::ParseError(format!("Failed to write: {}", e)))
}

fn draw_header(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
) -> crate::Result<()> {
  // Top border line
  let mut line = vec![' '; layout.line_width];
  for (i, _) in participants.iter().enumerate() {
    let pos = layout.participant_positions[i];
    line[pos - layout.participant_width / 2] = '┌';
    for j in 1..=layout.participant_width {
      line[pos - layout.participant_width / 2 + j] = '─';
    }
    line[pos + layout.participant_width / 2 + 1] = '┐';
  }
  write_line(out, &line)?;

  // Names line
  let mut line = vec![' '; layout.line_width];
  for (i, participant) in participants.iter().enumerate() {
    let pos = layout.participant_positions[i];
    let start = pos - layout.participant_width / 2 + 1;
    line[pos - layout.participant_width / 2] = '│';
    for (j, c) in participant.chars().enumerate() {
      line[start + j] = c;
    }
    line[pos + layout.participant_width / 2 + 1] = '│';
  }
  write_line(out, &line)?;

  // Bottom border with lifeline markers
  let mut line = vec![' '; layout.line_width];
  for (i, _) in participants.iter().enumerate() {
    let pos = layout.participant_positions[i];
    line[pos - layout.participant_width / 2] = '└';
    for j in 1..=layout.participant_width {
      line[pos - layout.participant_width / 2 + j] = '─';
    }
    line[pos] = '┬'; // Lifeline marker
    line[pos + layout.participant_width / 2 + 1] = '┘';
  }
  write_line(out, &line)?;

  Ok(())
}

fn draw_message_text(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
  from: &str,
  to: &str,
  message: &str,
) -> crate::Result<()> {
  let mut line = vec![' '; layout.line_width];

  let from_pos = layout.get_participant_pos(from, participants).unwrap_or(0);
  let to_pos = layout.get_participant_pos(to, participants).unwrap_or(0);

  let (left_pos, right_pos) = if from_pos < to_pos {
    (from_pos, to_pos)
  } else {
    (to_pos, from_pos)
  };

  // Draw lifelines
  if left_pos < line.len() {
    line[left_pos] = '│';
  }
  if right_pos < line.len() {
    line[right_pos] = '│';
  }

  // Draw message text in the middle
  let text_start = left_pos + 1;
  let text_end = right_pos;
  if text_start < text_end {
    let available = text_end - text_start;
    let trimmed = if message.len() > available {
      &message[..available.min(message.len())]
    } else {
      message
    };
    let msg_start = text_start + (available.saturating_sub(trimmed.len()) + 1) / 2;
    for (j, c) in trimmed.chars().enumerate() {
      if msg_start + j < right_pos {
        line[msg_start + j] = c;
      }
    }
  }

  write_line(out, &line)?;

  Ok(())
}

fn draw_message_arrow(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
  from: &str,
  to: &str,
) -> crate::Result<()> {
  let mut line = vec![' '; layout.line_width];

  let from_pos = layout.get_participant_pos(from, participants).unwrap_or(0);
  let to_pos = layout.get_participant_pos(to, participants).unwrap_or(0);

  let (left_pos, right_pos, going_right) = if from_pos < to_pos {
    (from_pos, to_pos, true)
  } else {
    (to_pos, from_pos, false)
  };

  // Draw lifelines
  if left_pos < line.len() {
    line[left_pos] = '│';
  }
  if right_pos < line.len() {
    line[right_pos] = '│';
  }

  // Draw arrow
  if going_right {
    for pos in left_pos + 1..right_pos {
      if pos < line.len() {
        line[pos] = '─';
      }
    }
    if right_pos > 0 && right_pos - 1 < line.len() {
      line[right_pos - 1] = '>';
    }
  } else {
    if left_pos + 1 < line.len() {
      line[left_pos + 1] = '<';
    }
    for pos in left_pos + 2..right_pos {
      if pos < line.len() && (pos - (left_pos + 2)) % 2 == 0 {
        line[pos] = '─';
      }
    }
  }

  write_line(out, &line)?;

  Ok(())
}

fn draw_empty_line(out: &mut dyn Write, layout: &LayoutCalculator, _participants: &[String]) -> crate::Result<()> {
  let mut line = vec![' '; layout.line_width];
  // Just draw the lifelines
  for pos in &layout.participant_positions {
    if *pos < line.len() {
      line[*pos] = '│';
    }
  }
  write_line(out, &line)?;
  Ok(())
}

fn draw_footer(
  out: &mut dyn Write,
  layout: &LayoutCalculator,
  participants: &[String],
) -> crate::Result<()> {
  // Top border of closing boxes (with lifeline markers)
  let mut line = vec![' '; layout.line_width];
  for i in 0..participants.len() {
    let pos = layout.participant_positions[i];
    line[pos - layout.participant_width / 2] = '┌';
    for j in 1..=layout.participant_width {
      line[pos - layout.participant_width / 2 + j] = '─';
    }
    line[pos] = '┴'; // Lifeline marker
    line[pos + layout.participant_width / 2 + 1] = '┐';
  }
  write_line(out, &line)?;

  // Names line (closing boxes)
  let mut line = vec![' '; layout.line_width];
  for (i, participant) in participants.iter().enumerate() {
    let pos = layout.participant_positions[i];
    let start = pos - layout.participant_width / 2 + 1;
    line[pos - layout.participant_width / 2] = '│';
    for (j, c) in participant.chars().enumerate() {
      line[start + j] = c;
    }
    line[pos + layout.participant_width / 2 + 1] = '│';
  }
  write_line(out, &line)?;

  // Bottom border
  let mut line = vec![' '; layout.line_width];
  for (i, _) in participants.iter().enumerate() {
    let pos = layout.participant_positions[i];
    line[pos - layout.participant_width / 2] = '└';
    for j in 1..=layout.participant_width {
      line[pos - layout.participant_width / 2 + j] = '─';
    }
    line[pos + layout.participant_width / 2 + 1] = '┘';
  }
  write_line(out, &line)?;

  Ok(())
}
