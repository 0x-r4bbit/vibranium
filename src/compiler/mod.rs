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

    let strategy_config = StrategyConfig {
      input_path: PathBuf::from(&self.config.project_path),
      output_path: artifacts_dir,
      smart_contract_sources: project_config.smart_contract_sources,
      compiler_options: config.compiler_options.clone(),
    };

    let compiler_strategy = match config.compiler.parse() {
      Ok(SupportedCompilers::Solc) => CompilerStrategy::new(Box::new(SolcStrategy::new(strategy_config))),
      Err(err) => {
        if config.compiler_options.is_empty() {
          return Err(err)
        }
        CompilerStrategy::new(Box::new(DefaultStrategy::new(&config.compiler, config.compiler_options)))
      },
    };

    compiler_strategy.execute().map_err(error::CompilerError::Io)
  }
}
