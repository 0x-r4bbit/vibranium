const LOCALHOST_ADDRESS: &str = "127.0.0.1";
const LOCALHOST_ALIAS: &str = "localhost";

pub fn merge_cli_options(a: Vec<String>, b: Vec<String>) -> Vec<String> {

  let mut merged = vec![];

  for (i, e) in a.iter().enumerate() {
    if !e.starts_with("--") {
      continue;
    }

    merged.push(e.to_owned());

    if let Some(next) = a.get(i+1) {
      if !next.starts_with("--") && b.contains(&e) {
        let ii = b.iter().position(|x| x == e).unwrap();
        if let Some(b_next) = b.get(ii+1) {
          if !b_next.starts_with("--") {
            merged.push(b_next.to_owned());
          }
        }
      } else {
        if !next.starts_with("--") {
          merged.push(next.to_owned())
        }
      }
    }
  }

  for e in b {
    if !merged.contains(&e) {
      merged.push(e.to_owned());
    }
  }

  merged
}

pub fn normalize_localhost(host: String) -> String {
  match host.as_ref() {
    LOCALHOST_ADDRESS | LOCALHOST_ALIAS => LOCALHOST_ADDRESS.to_owned(),
    _ => host
  }
}

#[cfg(test)]
mod tests {

  mod merge_cli_options {

    use super::super::merge_cli_options;

    #[test]
    fn it_should_merge_two_vec_of_options() {
      let a = vec!["--one".to_string(),"--two".to_string()];
      let b = vec!["--three".to_string(), "--four".to_string()];

      let merged = merge_cli_options(a, b);
      assert_eq!(merged, vec!["--one".to_string(), "--two".to_string(), "--three".to_string(), "--four".to_string()]);
    }

    #[test]
    fn it_should_keep_option_values_in_place() {
      let a = vec!["--one".to_string(),"value1".to_string(), "--three".to_string()];
      let b = vec!["--two".to_string(), "value2".to_string()];

      let merged = merge_cli_options(a, b);
      assert_eq!(merged, vec!["--one".to_string(), "value1".to_string(), "--three".to_string(), "--two".to_string(), "value2".to_string()]);
    }

    #[test]
    fn it_should_override_options() {
      let a = vec!["--one".to_string(),"value1".to_string(), "--two".to_string()];
      let b = vec!["--one".to_string(), "value2".to_string()];

      let merged = merge_cli_options(a, b);
      assert_eq!(merged, vec!["--one".to_string(), "value2".to_string(), "--two".to_string()]);
    }

    #[test]
    fn it_should_override_options_with_no_value_given() {
      let a = vec!["--one".to_string(),"value1".to_string(), "--two".to_string(), "value2".to_string()];
      let b = vec!["--one".to_string(), "value1".to_string(), "--two".to_string()];

      let merged = merge_cli_options(a, b);
      assert_eq!(merged, vec!["--one".to_string(), "value1".to_string(), "--two".to_string()]);
    }

    #[test]
    fn it_should_throw_out_duplicates_and_keep_the_last() {
      let a = vec!["--one".to_string(),"value1".to_string()];
      let b = vec!["--two".to_string(), "value1".to_string(), "--two".to_string(), "value2".to_string()];

      let merged = merge_cli_options(a, b);
      assert_eq!(merged, vec!["--one".to_string(), "value1".to_string(), "--two".to_string(), "value2".to_string()]);
    }
  }
}
