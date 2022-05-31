use bungee_backup::*;

const BAD_QUOTES_YAML: &str = "---
- name: Bad Quotes Yaml
  variable: \"somevar\"containing\"
  ";

#[test]
fn none_for_bad_quotes_yaml() {
    assert!(get_config(String::from(BAD_QUOTES_YAML)).is_none())
}
