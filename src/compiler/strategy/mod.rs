pub mod default;
pub mod solcjs;
pub mod solc;

use std::path::{PathBuf};
use std::process::{Child};

pub trait Compile {
  fn compile(&self) -> Result<Child, std::io::Error>;
}

pub struct StrategyConfig {
  pub input_path: PathBuf,
  pub output_path: PathBuf,
  pub smart_contract_sources: Vec<String>,
  pub compiler_options: Option<Vec<String>>,
}

pub struct CompilerStrategy<'a> {
  strategy: Box<Compile + 'a>,
}

impl<'a> CompilerStrategy<'a> {
  pub fn new(strategy: Box<Compile + 'a>) -> CompilerStrategy {
    CompilerStrategy {
      strategy
    }
  }

  pub fn execute(&self) -> Result<Child, std::io::Error> {
    self.strategy.compile()
  }
}
