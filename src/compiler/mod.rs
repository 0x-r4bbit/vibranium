pub mod error;

use std::process::{Command, Child};
use glob::glob;
use crate::config;

#[derive(Debug)]
pub struct CompilerConfig<'a> {
  pub compiler: String,
  pub compiler_options: Vec<&'a str>,
}

pub struct Compiler<'a> {
  config: &'a config::Config,
}

impl<'a> Compiler<'a> {
  pub fn new(config: &config::Config) -> Compiler {
    Compiler {
      config
    }
  }

  pub fn compile(&self, config: CompilerConfig) -> Result<Child, error::CompilerError> {
    let project_config = self.config.read().map_err(error::CompilerError::InvalidConfig)?;
    let artifacts_dir = self.config.project_path.join(&project_config.artifacts_dir);

    let mut args: Vec<String> = vec![
      "--ast-json".to_string(),
      "--abi".to_string(),
      "-o".to_string(),
      artifacts_dir.to_string_lossy().to_string(),
    ];

    for pattern in project_config.smart_contract_sources {
      let mut full_pattern = self.config.project_path.to_string_lossy().to_string();
      full_pattern.push_str(&pattern);
      for entry in glob(&full_pattern).unwrap().filter_map(Result::ok) {
        args.push(entry.to_string_lossy().to_string());
      }
    }

    args.push("--overwrite".to_string());

    Command::new(config.compiler)
      .args(args)
      .args(config.compiler_options)
      .spawn()
      .map_err(error::CompilerError::Io)
  }
}
