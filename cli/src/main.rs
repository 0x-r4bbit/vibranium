#[macro_use]
extern crate clap;
extern crate log;
extern crate env_logger;
extern crate vibranium;
extern crate toml;

use std::env;
use log::LevelFilter;
use std::process;
use std::path::PathBuf;
use std::io::{self, Write};

use clap::{App, SubCommand, Arg};

use vibranium::Vibranium;
use vibranium::blockchain::NodeConfig;
use vibranium::compiler::CompilerConfig;

mod error;

type Error = Box<std::error::Error>;

fn main() {
  if let Err(e) = run() {
    eprintln!("Aborted due to error:\n");
    eprintln!("{}", e);
    process::exit(1);
  }
}

fn run() -> Result<(), Error> {
  let mut app = App::new("Vibranium CLI")
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
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to Vibranium project from which to spin up a node")
                      .takes_value(true))
                    .arg(Arg::with_name("client-opts")
                      .value_name("OPTIONS")
                      .help("Specifies node specific options that will be passed down to the client")
                      .multiple(true)
                      .raw(true))
                    .arg(Arg::with_name("verbose")
                      .short("v")
                      .long("verbose")
                      .help("Generates verbose output"))
                  )
                  .subcommand(SubCommand::with_name("init")
                    .about("Initializes a Vibranium project inside the current directory, or a given path")
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to directory in which to initialize Vibranium project")
                      .takes_value(true))
                    .arg(Arg::with_name("verbose")
                      .short("v")
                      .long("verbose")
                      .help("Generates verbose output"))
                  )
                  .subcommand(SubCommand::with_name("reset")
                    .about("Resets Vibranium project inside the current directory, or a given path")
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to Vibranium project to reset")
                      .takes_value(true))
                    .arg(Arg::with_name("verbose")
                      .short("v")
                      .long("verbose")
                      .help("Generates verbose output"))
                  )
                  .subcommand(SubCommand::with_name("config")
                    .about("Reads and writes configuration options of a Vibranium project")
                    .arg(Arg::with_name("path")
                      .short("p")
                      .long("path")
                      .value_name("PATH")
                      .help("Specifies path to Vibranium project")
                      .takes_value(true))
                    .arg(Arg::with_name("set")
                      .number_of_values(2)
                      .value_names(&["CONFIG_OPTION", "VALUE"])
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
                    .arg(Arg::with_name("verbose")
                      .short("v")
                      .long("verbose")
                      .help("Generates verbose output"))
                  );

  let matches = app.clone().get_matches();

  if let (_, Some(cmd)) = matches.subcommand() {
    if cmd.is_present("verbose") {
      env_logger::Builder::from_default_env().filter(None, LevelFilter::Info).init();
    }
  }

  match matches.subcommand() {
    ("node", Some(cmd)) => {
      println!("Starting blockchain node...");
      let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
      let vibranium = Vibranium::new(path);

      let client_options = cmd.values_of("client-opts").map(|options| {
        options.map(std::string::ToString::to_string).collect()
      });

      let config = NodeConfig {
        client: cmd.value_of("client").map(std::string::ToString::to_string),
        client_options,
      };
    
      vibranium.start_node(config)?;
    },

    ("init", Some(cmd)) => {
      println!("Initializing empty Vibranium project...");
      let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
      let vibranium = Vibranium::new(path);

      vibranium.init_project().and_then(|_| {
        println!("Done.");
        Ok(())
      })?
    },

    ("reset", Some(cmd)) => {
      println!("Resetting Vibranium project...");
      let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
      let vibranium = Vibranium::new(path);
      vibranium.reset_project().and_then(|_| {
        println!("Done.");
        Ok(())
      })?
    },

    ("config", Some(cmd)) => {
      let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
      let vibranium = Vibranium::new(path);

      if let Some(options) = cmd.values_of("set") {
        let mut args: Vec<String> = options.map(std::string::ToString::to_string).collect();
        let config_option = args.remove(0);
        let mut value_arg = args[0].to_owned(); 

        let value = if is_multi_value_arg(&value_arg) {

          remove_multi_value_delimitiers(&mut value_arg);
          let values: Vec<String> = value_arg.split(',')
            .map(str::trim)
            .map(std::string::ToString::to_string)
            .collect();

          toml::value::Value::try_from(values)
            .map_err(error::CliError::ConfigurationSetError)?
        } else {
          toml::value::Value::try_from(value_arg)
            .map_err(error::CliError::ConfigurationSetError)?
        };

        vibranium.set_config(config_option, value)?
      }
    },

    ("compile", Some(cmd)) => {
      println!("Compiling Vibranium project...");
      let path = pathbuf_from_or_current_dir(cmd.value_of("path"))?;
      let vibranium = Vibranium::new(path);

      let compiler_options = cmd.values_of("compiler-opts").map(|options| {
        options.map(std::string::ToString::to_string).collect()
      });

      let config = CompilerConfig {
        compiler: cmd.value_of("compiler").map(std::string::ToString::to_string),
        compiler_options,
      };

      vibranium
        .compile(config)
        .map_err(error::CliError::CompilationError)
        .and_then(|output| {
          io::stdout().write_all(&output.stdout).unwrap();
          println!("Done.");
          Ok(())
        })?
    },

    _ => {
      app.print_help()?
    }
  };

  Ok(())
}

fn pathbuf_from_or_current_dir(path: Option<&str>) -> Result<PathBuf, std::io::Error> {
  path.map(|p| Ok(PathBuf::from(p))).unwrap_or_else(env::current_dir)
}

fn is_multi_value_arg(value: &str) -> bool {
  value.starts_with('[') && value.ends_with(']')
}

fn remove_multi_value_delimitiers(value: &mut String) {
  value.remove(0);
  value.pop();
}
