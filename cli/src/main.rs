#[macro_use]
extern crate clap;
extern crate vibranium;

use std::process::exit;
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
                    .arg(Arg::with_name("client-opts")
                      .value_name("OPTIONS")
                      .help("Specifies node specific options that will be passed down to the client")
                      .multiple(true)
                      .raw(true))
                  ).get_matches();

  if let ("node", Some(cmd)) = matches.subcommand() {
    let vibranium = Vibranium::new();

    let mut client_options = vec![];

    if let Some(options) = cmd.values_of("client-opts") {
      client_options = options.collect();
    }

    let config = NodeConfig {
      client: cmd.value_of("client").unwrap_or(DEFAULT_NODE_CLIENT),
      client_options: &client_options,
    };
  
    if let Err(err) = vibranium.start_node(config) {
      eprintln!("Error: {:?}", err);
      exit(1);
    }
  }
}
