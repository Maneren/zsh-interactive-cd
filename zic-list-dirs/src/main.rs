#![warn(clippy::pedantic)]

use std::{
  borrow::Cow,
  cmp::Ordering,
  env, fs,
  io::{self, Cursor},
  path::Path,
  process,
};

use regex::Regex;
use shell_escape::escape;
use skim::{
  prelude::{SkimItemReader, SkimOptionsBuilder},
  Skim,
};

fn main() {
  let mut args = env::args();
  args.next(); // skip first argument

  let lbuffer = args.next().unwrap_or_default();
  let lbuffer_expanded = args.next().unwrap_or_default();

  let mut tokens = lbuffer.split_whitespace();
  let mut tokens_expanded = lbuffer_expanded.split_whitespace();

  let cmd = match tokens.next() {
    Some(cmd) => cmd,
    _ => backup(),
  };

  if cmd != "cd" {
    backup();
  }

  let input = tokens.next().unwrap_or_default();
  let input_path = tokens_expanded.nth(1).unwrap_or_default();

  if input == "~" {
    print!("cd ~/");
    process::exit(0);
  }

  let input_path = if input_path.starts_with('~') {
    input_path.replace('~', &env::var("HOME").unwrap_or_default())
  } else {
    input_path.to_string()
  };

  let entries = match list_files(&input_path) {
    Ok(entries) => entries,
    _ => backup(),
  };

  let result = match entries.len() {
    0 => backup(),
    1 => entries[0].clone(),
    _ => skim(entries.join("\n")),
  };

  let result = format_result(input, &result);

  print!("cd {result}"); // main output

  process::exit(0);
}

fn format_result(input: &str, result: &str) -> String {
  let base = input
    .chars()
    .rev()
    .skip_while(|ch| ch != &'/')
    .collect::<String>()
    .chars()
    .rev()
    .collect::<String>(); // without the last part `path/to/fold` -> 'path/to`

  let result = escape(Cow::Borrowed(result)).to_string();

  if base.is_empty() {
    format!("{result}/")
  } else {
    format!("{base}{result}/")
  }
}

fn skim(input: String) -> String {
  let options = SkimOptionsBuilder::default()
    .height(Some("50%"))
    .multi(false)
    .reverse(true)
    .build()
    .unwrap();

  let item_reader = SkimItemReader::default();
  let items = item_reader.of_bufread(Cursor::new(input));

  let source = Some(items);
  let selected_items =
    Skim::run_with(&options, source).map_or_else(Vec::new, |out| out.selected_items);

  selected_items.get(0).unwrap().text().into_owned()
}

fn backup() -> ! {
  process::exit(1);
}

fn list_files(input: &str) -> io::Result<Vec<String>> {
  // constructs a regex and calls list_subdirs
  // if input is full path, then then return all subdirs
  // else try searching for subdirs that start with input
  // or for those that include the input as substring as a last resort

  // list_subdirs prefixes with '^' and suffixes with '.*$'

  // $zic_case_insensitive and $zic_ignore_dot applies to the search

  let ignore_dot = env::var("zic_ignore_dot") == Ok("true".to_string());

  let input = if input.starts_with('/') {
    input.to_string()
  } else {
    let current = env::current_dir()
      .expect("current dir error")
      .to_str()
      .unwrap()
      .to_owned();

    format!("{current}/{input}")
  };

  // if ends with /
  if input.ends_with('/') {
    let regex = if ignore_dot { "." } else { "[^.]" };
    return list_subdirs(&input, regex);
  }

  let (mut base, seg) = input.rsplit_once('/').unwrap_or((&input, "."));

  if base.is_empty() {
    base = "/";
  }

  let path = Path::new(&base);
  let dir = path.canonicalize().expect("dir err");
  let dir = dir.to_str().unwrap();

  // escape characters in the basename to be regex-safe
  // (can be bypassed, but with chars that can't be in filnames anyway)
  let mut escaped = regex_escape(seg);

  let mut regex = if ignore_dot {
    format!("[.]?{escaped}")
  } else {
    escaped.clone()
  };

  let starts_with_seg = list_subdirs(dir, &regex)?;

  if !starts_with_seg.is_empty() {
    return Ok(starts_with_seg);
  }

  // if first character of input ($1) is .,
  // force starting . in the regex
  if seg.starts_with('.') {
    escaped = regex_escape(&seg.chars().skip(1).collect::<String>());
    regex = format!("[.].*{escaped}");
  } else if ignore_dot {
    regex = format!(".*{escaped}");
  } else {
    regex = format!("[^.].*{escaped}");
  }

  list_subdirs(dir, &regex)
}

fn list_subdirs(base: &str, regex: &str) -> io::Result<Vec<String>> {
  let regex = if env::var("zic_case_insensitive") == Ok("true".to_string()) {
    format!("^(?i)({regex}.*)$")
  } else {
    format!("^{regex}.*$")
  };

  let final_regex = Regex::new(&regex).unwrap();

  let mut entries = vec![];

  for entry in fs::read_dir(Path::new(base)).expect("read dir error") {
    let entry = entry?;

    if let Ok(file_type) = entry.file_type() {
      if !file_type.is_dir() {
        continue;
      }

      let entry = entry.file_name().to_str().unwrap().to_string();

      if final_regex.is_match(&entry) {
        entries.push(entry);
      }
    }
  }

  entries.sort_unstable_by(|a, b| {
    let a_dot = a.starts_with('.');
    let b_dot = b.starts_with('.');

    match (a_dot, b_dot) {
      (true, false) => Ordering::Greater,
      (false, true) => Ordering::Less,
      _ => Ordering::Equal,
    }
  });

  Ok(entries)
}

fn regex_escape(input: &str) -> String {
  input
    .chars()
    .map(|ch| {
      if ch == '^' {
        "\\^".to_owned()
      } else {
        format!("[{ch}]")
      }
    })
    .collect()
}
