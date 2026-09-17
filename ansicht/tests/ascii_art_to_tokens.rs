use anyhow::Result;
use ansicht::reader::ascii_art::parse_tokens;
use pretty_assertions::assert_eq;

mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;

#[test]
fn oauth_happy_path() -> Result<()> {
  let content = read_example("sequence-diagram/oauth.happy_path.ascii")?;
  let tokens = parse_tokens(&content);

  assert_eq!(
    tokens,
    sequence_diagram_fixtures::oauth_happy_path_tokens()
  );
  Ok(())
}
