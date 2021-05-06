use anyhow::Result;
use asciidoctrine::{self, *};
use lisa::*;
use pretty_assertions::assert_eq;

#[test]
fn use_snippets() -> Result<()> {
  let content = r#"
We need the testmodule for this project.

[[sample1_required_modules]]
[source, lua]
----
require "testmodule"
----

This is the importing file. We could print out the version.

[source, lua, save]
.sample1.lua
----
<<sample1_required_modules>>

print(testmodule.version)
----
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::from_iter(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample1.lua").unwrap(),
  r#"require "testmodule"

print(testmodule.version)
"#
);


  assert!(outputs.is_empty()); // <1>

  Ok(())
}

#[test]
fn handle_snippet_order() -> Result<()> {
  let content = r#"
First we give a short outline of the program. It imports the required modules
and then prints out its version.

[source, lua, save]
.sample2.lua
----
<<sample2_required_modules>>

print(testmodule.version)
----

We need the testmodule for this project.

[[sample2_required_modules]]
[source, lua]
----
require "testmodule"
----
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::from_iter(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample2.lua").unwrap(),
  r#"require "testmodule"

print(testmodule.version)
"#
);


  assert!(outputs.is_empty()); // <1>

  Ok(())
}

#[test]
fn use_snippet_multiple_times() -> Result<()> {
  let content = r#"
Lets assume we want to use the following snippet in multiple places.

[[sample3_multiple]]
[source, lua]
----
require "testmodule"
----

Than we could import it in the same snippet multiple times.

[source, lua, save]
.sample3-1.lua
----
<<sample3_multiple>>

print(testmodule.version)

<<sample3_multiple>>
----

And we could even use it again in another snippet.

[source, lua, save]
.sample3-2.lua
----
<<sample3_multiple>>

print(testmodule.version .. "my other snippet")
----

"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::from_iter(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample3-1.lua").unwrap(),
  r#"require "testmodule"

print(testmodule.version)

require "testmodule"
"#
);
assert_eq!(
  outputs.remove("sample3-2.lua").unwrap(),
  r#"require "testmodule"

print(testmodule.version .. "my other snippet")
"#
);


  assert!(outputs.is_empty()); // <1>

  Ok(())
}

#[test]
fn append_snippets() -> Result<()> {
  let content = r#"
We do some thing in our code.

[source, lua, save]
.sample4.lua
----
<<some_process>>

print(result_of_someprocess)
----

To do this we need to do something with a variable.

[[some_process]]
[source, lua]
----
variable = 42
variable = variable + 42
----

But something else has also to be done. For exaple we need to set the result.

[[some_process]]
[source, lua]
----
result_of_someprocess = variable * variable
----

Now lets go on to another thing ...
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::from_iter(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample4.lua").unwrap(),
  r#"variable = 42
variable = variable + 42
result_of_someprocess = variable * variable

print(result_of_someprocess)
"#
);


  assert!(outputs.is_empty()); // <1>

  Ok(())
}

#[test]
fn append_snippets_with_customized_join() -> Result<()> {
  let content = r#"
Let's imagine we need some rust struct.

[[mystruct]]
[source, rust]
----
pub struct MyStruct { <<mystruct_fields|join=", ">> }
----

In our main process we need to define the struct an d initialize it.

[source, rust, save]
.sample5.rs
----
<<mystruct>>

impl MyStruct {
  pub fn new {
    MyStruct {
      <<init_fields|join=",\n">>
    }
  }
}
----

In our struct we have variable x

[[mystruct_fields]]
[source, rust]
----
x: String
----

And we initialize it properly

[[init_fields]]
[source, rust]
----
x: "this is the x text".to_string()
----

Now we can talk about all the functions that use x...

After some time we may have a function that use some other variable y.

[[mystruct_fields]]
[source, rust]
----
y: u8
----

And how is it initialized? You know the answer:

[[init_fields]]
[source, rust]
----
y: 42
----

And so on ...
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::from_iter(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample5.rs").unwrap(),
  r#"pub struct MyStruct { x: String, y: u8 }

impl MyStruct {
  pub fn new {
    MyStruct {
      x: "this is the x text".to_string(),
      y: 42
    }
  }
}
"#
);


  assert!(outputs.is_empty()); // <1>

  Ok(())
}

