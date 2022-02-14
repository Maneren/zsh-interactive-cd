use std::{cmp::Ordering, env, fs, io, path::Path};

use regex::Regex;

fn main() {









  
  let input = env::args().nth(1).unwrap_or_default();

  if let Ok(val) = list_files(&input) {
    let entries = val.join("\n");
    println!("{entries}")
  };
}

fn list_files(input: &str) -> io::Result<Vec<String>> {
  //  constructs a regex and calls list_subdirs
  //  if input is full path, then then return all subdirs
  // else try searching for subdirs that start with input
  // or for those that include the input as substring as a last resort

  // list_subdirs prefixes with '^' and suffixes with '.*$'

  // $zic_case_insensitive and $zic_ignore_dot applies to the search

  let ignore_dot = env::var("zic_ignore_dot") == Ok("true".to_string());

  let input = if input.starts_with('/') {
    input.to_owned()
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

  let (base, seg) = input.rsplit_once('/').unwrap_or((&input, "."));

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
        entries.push(entry)
      }
    }
  }

  entries.sort_unstable_by(|a, b| {
    let adot = a.starts_with('.');
    let bdot = b.starts_with('.');

    match (adot, bdot) {
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
