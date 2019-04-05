pub mod default;
pub mod solc;

use std::process::{Child};

pub trait Strategy {
  fn execute(&self) -> Result<Child, std::io::Error>;
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
