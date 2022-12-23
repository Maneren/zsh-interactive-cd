#![warn(clippy::pedantic)]

use std::{
  borrow::Cow,
  env, fs,
  io::{self, Cursor},
  path::Path,
  process,
};

use regex::RegexBuilder;
use shell_escape::escape;
use skim::{
  prelude::{Key, SkimItemReader, SkimOptionsBuilder},
  Skim,
};

struct Options {
  case_insensitive: bool,
  ignore_dot: bool,
}

fn main() {
  let mut args = env::args();
  args.next(); // skip first argument

  let lbuffer = args.next().unwrap_or_default();
  let lbuffer_expanded = args.next().unwrap_or_default();

  let (_, input) = lbuffer.split_once(' ').unwrap_or_default();
  let (_, input_path) = lbuffer_expanded.split_once(' ').unwrap_or_default();

  let is_env_enabled = |name| matches!(env::var(name), Ok(x) if matches!(x.as_str(), "true" | "1"));

  let options = Options {
    case_insensitive: is_env_enabled("zic_case_insensitive"),
    ignore_dot: is_env_enabled("zic_ignore_dot"),
  };

  let input_path = if input_path.starts_with("~/") {
    input_path.replace('~', &env::var("HOME").unwrap_or_default())
  } else {
    input_path.to_string()
  };

  let (base_path, search_term) = parse_path(&input_path);

  let base_path = if base_path.is_empty() {
    ".".into()
  } else {
    base_path
  };

  let base_path = Path::new(&base_path)
    .canonicalize()
    .unwrap_or_else(|_| abort())
    .to_string_lossy()
    .to_string();

  let subdirs = match get_subdirs(&base_path) {
    Ok(entries) => entries,
    _ => abort(),
  };

  let filtered = filter_dir_list(&search_term, &options, &subdirs);

  let result = match &filtered[..] {
    [] => {
      eprint!("\x07"); // ring a bell
      print!("cd {input}");
      exit();
    }
    [entry] => Some(entry.clone()),
    _ => {
      let mut sorted = filtered;

      sorted.sort_by_cached_key(|item| {
        levenshtein(&search_term, item.strip_prefix('.').unwrap_or(item))
      });

      skim(sorted.join("\n"))
    }
  };

  if let Some(result) = result {
    let (base_path, _) = parse_path(input);

    let result = format_result(&base_path, &result);

    print!("cd {result}"); // main output
  } else {
    print!("cd {input}"); // nothing was chosen
  }
}

fn parse_path(input_path: &str) -> (String, String) {
  if let Some((base, search)) = input_path.rsplit_once('/') {
    (format!("{base}/"), search.into())
  } else {
    (String::new(), input_path.into())
  }
}

fn exit() -> ! {
  process::exit(0);
}

fn abort() -> ! {
  process::exit(1);
}

fn get_subdirs(path: &String) -> io::Result<Vec<String>> {
  let mut subdirs = Vec::new();
  for entry in fs::read_dir(Path::new(path))? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      subdirs.push(path.file_name().unwrap().to_string_lossy().to_string());
    }
  }
  Ok(subdirs)
}

fn format_result(base: &str, result: &str) -> String {
  let result = escape(Cow::Borrowed(result)).to_string();

  format!("{base}{result}/") // base always ends with '/'
}

fn skim(input: String) -> Option<String> {
  let options = SkimOptionsBuilder::default()
    .height(Some("50%"))
    .multi(false)
    .reverse(true)
    .bind(vec!["esc:abort"])
    .build()
    .unwrap();

  let items = SkimItemReader::default().of_bufread(Cursor::new(input));

  let output = Skim::run_with(&options, Some(items)).expect("Failed to run Skim");

  match output.final_key {
    Key::Enter => output
      .selected_items
      .first()
      .map(|item| item.text().into_owned()),
    _ => None,
  }
}

fn filter_dir_list(search_term: &str, options: &Options, subdirs: &[String]) -> Vec<String> {
  // constructs a regex and calls the inner function
  // which prefixes it with '^' and suffixes with '.*$'
  // $zic_case_insensitive and $zic_ignore_dot applies to the search
  if search_term.is_empty() {
    return if options.ignore_dot {
      subdirs.to_vec()
    } else {
      subdirs
        .iter()
        .filter(|dir| !dir.starts_with('.'))
        .cloned()
        .collect()
    };
  }

  let escaped = regex_escape(search_term);

  let regex = if options.ignore_dot {
    format!("[.]?{escaped}")
  } else {
    escaped.to_string()
  };

  let starts_with_search = filter_dir_list_inner(&regex, options, subdirs);

  if !starts_with_search.is_empty() {
    return starts_with_search;
  }

  // if first character of search_term is .,
  // force a starting . in the regex
  let regex = if let Some(without_prefix) = escaped.strip_prefix("[.]") {
    format!("[.].*{without_prefix}")
  } else if options.ignore_dot {
    format!(".*{escaped}")
  } else {
    format!("[^.].*{escaped}")
  };

  let substring = filter_dir_list_inner(&regex, options, subdirs);

  if !substring.is_empty() {
    return substring;
  }

  // semi-fuzzy search
  let regex = regex.replace("][", "].*[");

  let semi = filter_dir_list_inner(&regex, options, subdirs);

  if !semi.is_empty() {
    return semi;
  }

  subdirs.to_vec()
}

fn filter_dir_list_inner(regex: &str, options: &Options, subdirs: &[String]) -> Vec<String> {
  let regex = format!("^{regex}.*$",);

  let final_regex = RegexBuilder::new(&regex)
    .case_insensitive(options.case_insensitive)
    .build()
    .expect("Invalid Regex");

  subdirs
    .iter()
    .filter(|entry| final_regex.is_match(entry))
    .cloned()
    .collect::<Vec<_>>()
}

fn regex_escape(input: &str) -> String {
  // escape characters in the basename to be regex-safe
  // (can be bypassed, but with chars that can't be in filnames anyway)
  input
    .chars()
    .map(|ch| match ch {
      '^' => r#"[\^]"#.to_owned(),
      '\\' => r#"[\\]"#.to_owned(),
      '[' => r#"[\[]"#.to_owned(),
      ']' => r#"[\]]"#.to_owned(),
      _ => format!("[{ch}]"),
    })
    .collect()
}

// optimized levenshtein distance algorithm
//
// stores only one row, filed to left and field to top
// skips first column as it follows a simple ascending sequence
fn levenshtein(a: &str, b: &str) -> usize {
  // short circuit if equal
  if a == b {
    return 0;
  }

  // use chars::count to prevent weird unicode issues
  let a_len = a.chars().count();
  let b_len = b.chars().count();

  // short circuit if empty
  if a.is_empty() {
    return b_len;
  }
  if b.is_empty() {
    return a_len;
  }

  // the loop reads a field from here (previous row)
  // and then sets the field to the current row's value

  // initialize as first row - ascending numbers
  let mut cache = (1..=b_len).collect::<Vec<_>>();

  let mut left = 0;

  for (i, a) in a.chars().enumerate() {
    let mut diagonal = i; // first column == 0..=b_len

    for (j, b) in b.chars().enumerate() {
      let m1 = diagonal + usize::from(a != b); // substitute
      let m2 = cache[j] + 1; // delete
      let m3 = left + 1; // insert

      diagonal = cache[j]; // cache[j] = up

      cache[j] = m1.min(m2).min(m3); // store best option

      left = cache[j]; // cache[j] = current
    }
  }

  left
}

#[cfg(test)]
mod tests {
  use std::string::ToString;

  use super::*;

  #[test]
  fn test_format_result() {
    assert_eq!(format_result("/home/", "user"), "/home/user/");
    assert_eq!(format_result("/", "home"), "/home/");
    assert_eq!(format_result("~/", "folder"), "~/folder/");
    assert_eq!(format_result("~/", "folder"), "~/folder/");
    assert_eq!(format_result("../", "folder"), "../folder/");
    assert_eq!(format_result("/home/", "user"), "/home/user/");
  }

  #[test]
  fn test_parse_path() {
    assert_eq!(parse_path("/home/use"), ("/home/".into(), "use".into()));
    assert_eq!(parse_path("/hom"), ("/".into(), "hom".into()));
    assert_eq!(
      parse_path("/home/user/"),
      ("/home/user/".into(), String::new())
    );
    assert_eq!(parse_path("/"), ("/".into(), String::new()));
    assert_eq!(parse_path("home/use"), ("home/".into(), "use".into()));
    assert_eq!(parse_path("home/"), ("home/".into(), String::new()));
    assert_eq!(
      parse_path("home/user/"),
      ("home/user/".into(), String::new())
    );
    assert_eq!(parse_path("home/"), ("home/".into(), String::new()));
  }

  #[test]
  fn test_filter_dir_list() {
    let dirs = [
      ".etc",
      ".home",
      ".lib",
      ".lib64",
      ".MNT",
      ".PROC",
      "bin",
      "boot",
      "lost+found",
      "root",
      "Run",
      "sbin",
      "srv",
      "sys",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect::<Vec<_>>();

    let options = Options {
      ignore_dot: false,
      case_insensitive: false,
    };

    assert_eq!(
      filter_dir_list("", &options, &dirs),
      vec![
        "bin",
        "boot",
        "lost+found",
        "root",
        "Run",
        "sbin",
        "srv",
        "sys",
      ]
    );
    assert_eq!(
      filter_dir_list(".", &options, &dirs),
      vec![".etc", ".home", ".lib", ".lib64", ".MNT", ".PROC",]
    );
    assert_eq!(filter_dir_list("b", &options, &dirs), vec!["bin", "boot",]);
    assert_eq!(filter_dir_list("oo", &options, &dirs), vec!["boot", "root"]);
    assert_eq!(filter_dir_list("in", &options, &dirs), vec!["bin", "sbin"]);
    assert_eq!(filter_dir_list("ib", &options, &dirs), Vec::<String>::new());
    assert_eq!(filter_dir_list("r", &options, &dirs), vec!["root"]);
    assert_eq!(
      filter_dir_list("mnt", &options, &dirs),
      Vec::<String>::new()
    );

    let options = Options {
      ignore_dot: true,
      case_insensitive: true,
    };

    assert_eq!(
      filter_dir_list("", &options, &dirs),
      vec![
        ".etc",
        ".home",
        ".lib",
        ".lib64",
        ".MNT",
        ".PROC",
        "bin",
        "boot",
        "lost+found",
        "root",
        "Run",
        "sbin",
        "srv",
        "sys",
      ]
    );
    assert_eq!(
      filter_dir_list(".", &options, &dirs),
      vec![".etc", ".home", ".lib", ".lib64", ".MNT", ".PROC",]
    );
    assert_eq!(filter_dir_list("r", &options, &dirs), vec!["root", "Run"]);
    assert_eq!(filter_dir_list("mnt", &options, &dirs), vec![".MNT"]);
    assert_eq!(filter_dir_list("m", &options, &dirs), vec![".MNT"]);
    assert_eq!(
      filter_dir_list("ib", &options, &dirs),
      vec![".lib", ".lib64"]
    );
  }
}
