#[macro_use]
extern crate clap;
extern crate vibranium;

use clap::{App, SubCommand, Arg};
use vibranium::Vibranium;
use vibranium::blockchain::NodeConfig;

const DEFAULT_NODE_CLIENT: &str = "parity";

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

  if let ("node", Some(cmd)) = matches.subcommand() {
    let vibranium = Vibranium::new();

    let config = NodeConfig {
      client: cmd.value_of("client").unwrap_or(DEFAULT_NODE_CLIENT),
    };
  
    vibranium.start_node(config);
  }
}
