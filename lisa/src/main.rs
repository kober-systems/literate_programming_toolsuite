use anyhow::Result;
use asciidoctrine::*;
use lisa::*;

fn run_lisa<'a>(_opts: &'a options::Opts, ast: AST<'a>) -> Result<AST<'a>> {
  let mut lisa = Lisa::new();
  Ok(lisa.transform(ast)?)
}

fn main() -> Result<()> {
  cli_template::cli_template(run_lisa)
}
