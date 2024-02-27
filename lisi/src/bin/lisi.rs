use anyhow::Result;
use asciidoctrine::*;
use lisi::*;

fn run_lisi<'a>(_opts: &'a options::Opts, ast: AST<'a>) -> Result<AST<'a>> {
  let mut lisi = Lisi::new();
  Ok(lisi.transform(ast)?)
}

fn main() -> Result<()> {
  cli_template::cli_template(run_lisi)
}
