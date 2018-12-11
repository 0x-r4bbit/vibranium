extern crate clap;
extern crate toml;

use clap::App;

fn main() {
  let pkg: toml::Value = toml::from_str(include_str!("../Cargo.toml")).unwrap();
  let version = pkg["package"]["version"].as_str().expect("'version' has to be a string!");

  App::new("Vibranium CLI")
          .version(version)
          .author("Pascal Precht <pascal.precht@gmail.com>")
          .about("Building DApps made easy")
          .get_matches();
}
