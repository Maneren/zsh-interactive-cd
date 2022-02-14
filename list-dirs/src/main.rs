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
  let lbuffer = env::args().nth(1).unwrap_or_default();

  if lbuffer == "cd" {
    backup();
  };

  let mut tokens = lbuffer.split_whitespace();

  let cmd = match tokens.next() {
    Some(cmd) => cmd,
    _ => return backup(),
  };

  if cmd != "cd" {
    backup();
  }

  let input = tokens.next().unwrap_or_default();

  // let start = Instant::now();

  let entries = match list_files(input) {
    Ok(entries) => entries,
    _ => return backup(),
  };

  // let elapsed = start.elapsed();
  // eprint!("{elapsed:?}");

  let result = match entries.len() {
    // 0 => return backup(),
    1 => entries[0].clone(),
    _ => skim(entries.join("\n")),
  };

  let result = format_result(input, &result);
  print!("cd {result}");

  process::exit(0);
}

fn format_result(input: &str, result: &str) -> String {
  // if user enters 'path/to/fold' remove the 'fold'
  // so 'folder' can be just simply appended later
  let base = if input.ends_with('/') {
    input
  } else {
    let mut base = input.rsplit_once('/').unwrap_or(("", "")).0;

    if base.is_empty() {
      base = "/";
    }

    base
  };

  let result = escape(Cow::Borrowed(result)).to_string();

  if base.is_empty() {
    format!("{result}/")
  } else {
    let base = escape(Cow::Borrowed(base)).to_string().replace("'~'", "~");

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

  // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
  // `SkimItem` was implemented for `AsRef<str>` by default
  let item_reader = SkimItemReader::default();
  let items = item_reader.of_bufread(Cursor::new(input));

  // `run_with` would read and show items from the stream
  let source = Some(items);
  let selected_items =
    Skim::run_with(&options, source).map_or_else(Vec::new, |out| out.selected_items);

  selected_items.get(0).unwrap().text().into_owned()
}

fn backup() {
  process::exit(1);
}

fn list_files(input: &str) -> io::Result<Vec<String>> {
  //  constructs a regex and calls list_subdirs
  //  if input is full path, then then return all subdirs
  // else try searching for subdirs that start with input
  // or for those that include the input as substring as a last resort

  // list_subdirs prefixes with '^' and suffixes with '.*$'

  // $zic_case_insensitive and $zic_ignore_dot applies to the search

  let ignore_dot = env::var("zic_ignore_dot") == Ok("true".to_string());

  let input = input.replace('~', &env::var("HOME").unwrap());

  let input = if input.starts_with('/') {
    input
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
