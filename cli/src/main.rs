extern crate clap;
extern crate toml;

use clap::App;

fn main() {
  let pkg: toml::Value = toml::from_str(include_str!("../Cargo.toml")).unwrap();
  let version = pkg["package"]["version"].as_str().unwrap();
  let authors = pkg["package"]["authors"].as_array().unwrap();

  App::new("Vibranium CLI")
          .version(version)
          .author(authors[0].as_str().unwrap())
          .about("Building DApps made easy")
          .get_matches();
}
