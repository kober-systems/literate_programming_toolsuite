use anyhow::Result;
use asciidoctrine::{self, *};
use clap::Parser;
use lisa::*;
use pretty_assertions::assert_eq;

#[test]
fn indented_snippets_with_params() -> Result<()> {
  let content = r#"
We have a snippet

[[inner_snippet]]
[source, yaml]
----
this is my snippet <<text>>
----

And we indent it

[source, yaml, save]
.sample.yaml
----
category:
    <<inner_snippet|
        text:="my param text">>

category2:
    <<inner_snippet|
        text:=<<referenced_param>> >>
----

And here we have a param we reference

[[referenced_param]]
[source, yaml]
----
referenced param text
----
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample.yaml").unwrap(),
  r#"category:
    this is my snippet my param text

category2:
    this is my snippet referenced param text
"#
);


  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn deep_nested_snippets_with_params() -> Result<()> {
  let content = r#"
We have a basic snippet

[[snippet_with_param]]
[source, python]
----
print("<<echo_text>>")
----

And we have a snippet that uses it.

[[calling_snippet_one]]
[source, python]
----
def my_function():
  <<snippet_with_param>>
  # <<echo_text>>
----

We have onther snippet that uses it too. But it overwrites the parameter.

[[calling_snippet_two]]
[source, python]
----
def my_other_function():
  <<snippet_with_param| echo_text := <<echo_text>> >>
  # <<echo_text>>
----

When we use these snippets we expect them to do different things. The
first should have the nested inner snippet untouched and the second
should use the parameter in the nested snippet.

[source, python, save]
.nested.py
----
<<calling_snippet_one|
    echo_text:="touch only outer">>

<<calling_snippet_two|
    echo_text:="touch inner snippet too">>
----

The default text is [[echo_text]]`untouched`.

"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("nested.py").unwrap(),
  r#"def my_function():
  print("untouched")
  # touch only outer

def my_other_function():
  print("touch inner snippet too")
  # touch inner snippet too
"#
);


  assert!(outputs.is_empty());

  Ok(())
}

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
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

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
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

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
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

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

But something else has also to be done. For example we need to set the
result.

[[some_process]]
[source, lua]
----
result_of_someprocess = variable * variable
----

Now lets go on to another thing ...
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

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

In our main process we need to define the struct and initialize it.

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
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn use_inline_snippets() -> Result<()> {
  let content = r#"
Let's imagine we need some rust struct.

[[mystruct]]
[source, rust]
----
pub struct MyStruct { <<mystruct_fields|join=", ">> }
----

In our main process we need to define the struct and initialize it.

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

In our struct we have variable [[mystruct_fields]]`x: String`. And we
initialize it properly

[[init_fields]]
[source, rust]
----
x: "this is the x text".to_string()
----

Now we can talk about all the functions that use x...

After some time we may have a function that use some other variable
[[mystruct_fields]]`y: u8`. And how is it initialized? You know the
answer:

[[init_fields]]
[source, rust]
----
y: 42
----

And so on ...
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
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


  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn indented_snippets() -> Result<()> {
  let content = r#"
Imagine you want to print a long pattern of "//***//" around so text to emphasize it.

We can do this:

[[print_pattern_once]]
[source, c]
----
printf("/");
printf("***");
printf("/");
----

But we want this line to be long

[[print_pattern]]
[source, c]
----
for (i=0;i<5;i++) {
  <<print_pattern_once>>
}
print("\n");
----

And now lets emphasize the text.

[source, c, save]
.sample7.c
----
<<print_pattern>>
print("My emphasized text!!\n"
<<print_pattern>>
----
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample7.c").unwrap(),
  r#"for (i=0;i<5;i++) {
  printf("/");
  printf("***");
  printf("/");
}
print("\n");
print("My emphasized text!!\n"
for (i=0;i<5;i++) {
  printf("/");
  printf("***");
  printf("/");
}
print("\n");
"#
);


  assert!(outputs.is_empty());

  Ok(())
}

#[test]
fn snippets_with_params() -> Result<()> {
  let content = r#"
There is a function we want to use in different contexts.

[[test_condition]]
[source, sh]
----
if [[ <<condition>> ]] then
  echo "<<err_message| condition:=<<condition>> >>"
  exit <<exit_code>>
fi
----

Normally we exit with the message
[[err_message]]`the condition <<condition>> was not met` and return exit
code [[exit_code]]`1`.

Now we can use this snippet to test some condition before we execute our
script. Lets say we want to make sure `file_xy.txt` exists.

[[checks]]
[source, sh]
----
<<test_condition| condition:="-f file_xy.txt">>
----

But we could also override the default snippets with a custom one. For
example to change the error message.

[[checks]]
[source, sh]
----
<<test_condition|condition:="-f file_yz.txt",
                 err_message:="my custom err message yz">>
----

It's also possible to nest snippets. We can just reference them. Let's
say we would like to return the message
[[custom_err_message]]`return from nested param snippet with code <<custom_exit_code>>`
and exit code [[custom_exit_code]]`42`.

[[checks]]
[source, sh]
----
<<test_condition|condition:="-f nested_params.txt",
                 err_message:=<<custom_err_message>>,
                 exit_code:=<<custom_exit_code>>>>
----

Now we put all of these conditions at the beginning of our script.

[source, sh, save]
.sample8.sh
----
<<checks>>

echo "you passed all checks"
----
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut lisa = Lisa::from_env(env);
  let _ast = lisa.transform(ast)?;

  // TODO ast vergleichen

  let mut outputs = lisa.into_cache().unwrap();

  assert_eq!(
  outputs.remove("sample8.sh").unwrap(),
  r#"if [[ -f file_xy.txt ]] then
  echo "the condition -f file_xy.txt was not met"
  exit 1
fi
if [[ -f file_yz.txt ]] then
  echo "my custom err message yz"
  exit 1
fi
if [[ -f nested_params.txt ]] then
  echo "return from nested param snippet with code 42"
  exit 42
fi

echo "you passed all checks"
"#
);


  assert!(outputs.is_empty());

  Ok(())
}

