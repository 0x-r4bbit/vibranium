#[macro_use]
extern crate clap;

use clap::App;

fn main() {
  let matches = App::new("Vibranium CLI")
                  .version(crate_version!())
                  .author(crate_authors!())
                  .about("Building DApps made easy")
                  .get_matches();
}
