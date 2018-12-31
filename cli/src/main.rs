#[macro_use]
extern crate clap;
extern crate vibranium;

use clap::{App, SubCommand, Arg};
use vibranium::Vibranium;

fn main() {
  let matches = App::new("Vibranium CLI")
                  .version(crate_version!())
                  .author(crate_authors!())
                  .about("Building DApps made easy")
                  .subcommand(SubCommand::with_name("node")
                    .about("Controls blockchain node")
                    .arg(Arg::with_name("client")
                      .short("c")
                      .long("client")
                      .value_name("CLIENT_BINARY")
                      .help("Specifies client used to start local Ethereum node")
                      .takes_value(true))
                  ).get_matches();

  if let Some(_matches) = matches.subcommand_matches("node") {
    let vibranium = Vibranium::new();
    vibranium.start_node();
  }
}
