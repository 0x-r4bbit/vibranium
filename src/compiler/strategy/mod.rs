pub mod default;
pub mod solcjs;
pub mod solc;

use std::path::{PathBuf};
use std::process::{Child};

pub trait Strategy {
  fn execute(&self) -> Result<Child, std::io::Error>;
}

pub struct StrategyConfig<'a> {
  pub input_path: PathBuf,
  pub output_path: PathBuf,
  pub smart_contract_sources: Vec<String>,
  pub compiler_options: Vec<&'a str>,
}

pub struct CompilerStrategy<'a> {
  strategy: Box<Strategy + 'a>,
}

impl<'a> CompilerStrategy<'a> {
  pub fn new(strategy: Box<Strategy + 'a>) -> CompilerStrategy {
    CompilerStrategy {
      strategy
    }
  }

  pub fn execute(&self) -> Result<Child, std::io::Error> {
    self.strategy.execute()
  }
}
