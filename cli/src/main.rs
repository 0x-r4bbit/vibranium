#[macro_use]
extern crate clap;
extern crate vibranium;

use std::env;
use std::process;
use std::path::PathBuf;
use std::io::{self, Write};

use clap::{App, SubCommand, Arg};

use vibranium::Vibranium;
use vibranium::blockchain::NodeConfig;
use vibranium::compiler::CompilerConfig;

mod error;

const DEFAULT_NODE_CLIENT: &str = "parity";

type Error = Box<std::error::Error>;

fn main() {
  if let Err(e) = run() {
    eprintln!("Aborted due to error:\n");
    eprintln!("{}", e);
    process::exit(1);
  } else {
    println!("Done.");
  }
}

fn run() -> Result<(), Error> {
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
                  )
                  .subcommand(SubCommand::with_name("init")
                    .about("Initializes a Vibranium project inside the current directory, or a given path")
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to directory in which to initialize Vibranium project")
                      .takes_value(true))
                  )
                  .subcommand(SubCommand::with_name("reset")
                    .about("Resets Vibranium project inside the current directory, or a given path")
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to Vibranium project to reset")
                      .takes_value(true))
                  )
                  .subcommand(SubCommand::with_name("compile")
                    .about("Compiles Smart Contracts from Vibranium project")
                    .arg(Arg::with_name("compiler")
                      .short("c")
                      .long("compiler")
                      .value_name("COMPILER_BINARY")
                      .help("Specifies compiler used to compile Smart Contracts")
                      .takes_value(true))
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to Vibranium project to compile")
                      .takes_value(true))
                    .arg(Arg::with_name("compiler-opts")
                      .value_name("OPTIONS")
                      .help("Specifies compiler specific options that will be passed down to the compiler")
                      .multiple(true)
                      .raw(true))
                  ).get_matches();

  if let ("node", Some(cmd)) = matches.subcommand() {
    println!("Starting blockchain node...");
    let vibranium = Vibranium::new(env::current_dir()?);

    let client = cmd.value_of("client").unwrap_or(DEFAULT_NODE_CLIENT);
    let mut client_options = vec![];

    if let Some(options) = cmd.values_of("client-opts") {
      client_options = options.collect();
    }

    let config = NodeConfig {
      client: &client,
      client_options: &client_options,
    };
  
    vibranium.start_node(config)?;
  }

  if let ("init", Some(cmd)) = matches.subcommand() {
    println!("Initializing empty Vibranium project...");
    let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
    let vibranium = Vibranium::new(path);

    vibranium.init_project()?
  }

  if let ("reset", Some(cmd)) = matches.subcommand() {
    println!("Resetting Vibranium project...");
    let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
    let vibranium = Vibranium::new(path);
    vibranium.reset_project()?
  }

  if let("compile", Some(cmd)) = matches.subcommand() {
    println!("Compiling Vibranium project...");
    let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
    let vibranium = Vibranium::new(path);

    let compiler_options = cmd.values_of("compiler-opts").map(|options| {
      options.map(|val| val.to_string()).collect()
    });

    let config = CompilerConfig {
      compiler: cmd.value_of("compiler").map(|cmd| cmd.to_string()),
      compiler_options,
    };

    vibranium
      .compile(config)
      .map_err(error::CliError::CompilationError)
      .and_then(|output| {
        io::stdout().write_all(&output.stdout).unwrap();
        Ok(())
      })?
  }

  Ok(())
}

fn pathbuf_from_or_current_dir(path: Option<&str>) -> Result<PathBuf, std::io::Error> {
  path.map(|p| Ok(PathBuf::from(p))).unwrap_or_else(env::current_dir)
}

