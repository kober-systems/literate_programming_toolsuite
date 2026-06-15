use std::{
  fs,
  io::{self, Write},
  path::PathBuf,
};

use ansicht::{reader, writer, Reader, Writer, AST};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about = "Convert ansicht diagram formats")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
  /// Convert between supported diagram formats.
  Convert(ConvertArgs),
}

#[derive(Debug, Parser)]
struct ConvertArgs {
  /// Input file. Reads from stdin when omitted.
  input: Option<PathBuf>,

  /// Output file. Writes to stdout when omitted.
  #[arg(short, long)]
  output: Option<PathBuf>,

  /// Input format. Inferred from the input file extension when omitted.
  #[arg(short, long, value_enum)]
  from: Option<Format>,

  /// Output format. Inferred from the output file extension when omitted.
  #[arg(short, long, value_enum)]
  to: Option<Format>,

  /// Feature name used when writing Cucumber/Gherkin.
  #[arg(long, default_value = "Interactions")]
  feature_name: String,

  /// Participant perspective used when writing Cucumber/Gherkin.
  #[arg(long, default_value = "Client")]
  perspective: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
  #[value(alias = "ascii")]
  AsciiArt,
  Mermaid,
  #[value(alias = "gherkin", alias = "feature")]
  Cucumber,
  Json,
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Convert(args) => convert(args),
  }
}

fn convert(args: ConvertArgs) -> anyhow::Result<()> {
  let input = read_input(args.input.as_ref())?;
  let from = args
    .from
    .or_else(|| args.input.as_ref().and_then(|path| infer_format(path)))
    .ok_or_else(|| anyhow::anyhow!("input format is required when it cannot be inferred"))?;
  let to = args
    .to
    .or_else(|| args.output.as_ref().and_then(|path| infer_format(path)))
    .ok_or_else(|| anyhow::anyhow!("output format is required when it cannot be inferred"))?;

  let ast = parse_ast(from, &input)?;
  let output = write_ast(to, ast, &args.feature_name, &args.perspective)?;

  write_output(args.output.as_ref(), &output)?;

  Ok(())
}

fn read_input(path: Option<&PathBuf>) -> anyhow::Result<String> {
  match path {
    Some(path) => Ok(fs::read_to_string(path)?),
    None => Ok(io::read_to_string(io::stdin())?),
  }
}

fn write_output(path: Option<&PathBuf>, output: &[u8]) -> anyhow::Result<()> {
  match path {
    Some(path) => Ok(fs::write(path, output)?),
    None => Ok(io::stdout().write_all(output)?),
  }
}

fn parse_ast<'a>(format: Format, input: &'a str) -> anyhow::Result<AST<'a>> {
  match format {
    Format::AsciiArt => Ok(reader::AsciiArtReader::new().parse(input)),
    Format::Mermaid => Ok(reader::MermaidReader::new().parse(input)?),
    Format::Cucumber => Ok(reader::CucumberReader::new().parse(input)?),
    Format::Json => anyhow::bail!("JSON is only supported as an output format"),
  }
}

fn write_ast(
  format: Format,
  ast: AST<'_>,
  feature_name: &str,
  perspective: &str,
) -> anyhow::Result<Vec<u8>> {
  let mut output = Vec::new();

  match format {
    Format::AsciiArt => writer::ascii_art::AsciiArtWriter::new().write(ast, &mut output)?,
    Format::Mermaid => writer::mermaid::MermaidWriter.write(ast, &mut output)?,
    Format::Cucumber => writer::cucumber::CucumberWriter {
      feature_name: feature_name.to_string(),
      perspective: perspective.to_string(),
    }
    .write(ast, &mut output)?,
    Format::Json => writer::json::JsonWriter.write(ast, &mut output)?,
  }

  Ok(output)
}

fn infer_format(path: &PathBuf) -> Option<Format> {
  let extension = path.extension()?.to_str()?.to_ascii_lowercase();

  match extension.as_str() {
    "ascii" | "txt" => Some(Format::AsciiArt),
    "mmd" | "mermaid" => Some(Format::Mermaid),
    "feature" | "gherkin" => Some(Format::Cucumber),
    "json" => Some(Format::Json),
    _ => None,
  }
}
