pub mod error;
mod strategy;
mod support;

use std::process::{Child};
use std::path::{PathBuf};
use crate::config;
use strategy::{CompilerStrategy, StrategyConfig};
use strategy::solc::SolcStrategy;
use strategy::solcjs::SolcJsStrategy;
use strategy::default::DefaultStrategy;
use support::SupportedCompilers;

#[derive(Debug)]
pub struct CompilerConfig {
  pub compiler: Option<String>,
  pub compiler_options: Option<Vec<String>>,
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
    let artifacts_dir = self.config.project_path.join(&project_config.sources.artifacts);

    let compiler = config.compiler.unwrap_or_else(|| {
      match &project_config.compiler {
        Some(config) => config.cmd.clone().unwrap_or_else(|| SupportedCompilers::Solc.to_string()),
        None => SupportedCompilers::Solc.to_string(),
      }
    });

    let compiler_options = match &config.compiler_options {
      Some(options) => Some(options.to_vec()),
      None => {
        match project_config.compiler {
          Some(config) => config.options,
          None => None
        }
      }
    };

    let strategy_config = StrategyConfig {
      input_path: PathBuf::from(&self.config.project_path),
      output_path: artifacts_dir,
      smart_contract_sources: project_config.sources.smart_contracts,
      compiler_options: compiler_options.clone()
    };

    let compiler_strategy = match compiler.parse() {
      Ok(SupportedCompilers::Solc) => CompilerStrategy::new(Box::new(SolcStrategy::new(strategy_config))),
      Ok(SupportedCompilers::SolcJs) => CompilerStrategy::new(Box::new(SolcJsStrategy::new(strategy_config))),
      Err(err) => {
        if compiler_options.is_none() {
          return Err(err)
        }
        CompilerStrategy::new(Box::new(DefaultStrategy::new(compiler.to_owned(), compiler_options.unwrap())))
      },
    };

    compiler_strategy.execute().map_err(error::CompilerError::Io)
  }
}
