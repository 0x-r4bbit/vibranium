pub mod error;
mod strategy;

use std::process::{Child};
use std::path::{PathBuf};
use std::str::FromStr;
use std::string::ToString;
use crate::config;
use strategy::CompilerStrategy;
use strategy::solc::{SolcStrategy, SolcStrategyConfig, SOLC_COMPILER_BINARY};

pub enum SupportedCompilers {
  Solc,
}

impl FromStr for SupportedCompilers {
  type Err = error::CompilerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      SOLC_COMPILER_BINARY => return Ok(SupportedCompilers::Solc),
      _ => Err(error::CompilerError::UnsupportedStrategy),
    }
  }
}

impl ToString for SupportedCompilers {
  fn to_string(&self) -> String {
    match self {
      SupportedCompilers::Solc => SOLC_COMPILER_BINARY.to_string(),
    }
  }
}

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

    let compiler_strategy = match config.compiler.parse() {
      Ok(SupportedCompilers::Solc) => {
        CompilerStrategy::new(Box::new(SolcStrategy::new(SolcStrategyConfig {
          input_path: PathBuf::from(&self.config.project_path),
          output_path: artifacts_dir,
          smart_contract_sources: project_config.smart_contract_sources,
          compiler_options: config.compiler_options,
        })))
      },
      Err(err) => return Err(err),
    };

    compiler_strategy.execute().map_err(error::CompilerError::Io)
  }
}
