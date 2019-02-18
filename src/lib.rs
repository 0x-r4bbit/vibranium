pub mod blockchain;
pub mod code_generator;

use std::process::ExitStatus;
use std::path::PathBuf;

#[macro_use]
extern crate serde_derive;

#[derive(Debug)]
pub struct Vibranium;

impl Vibranium {
  pub fn new() -> Vibranium {
    Vibranium {
    }
  }

  pub fn start_node(&self, config: blockchain::NodeConfig) -> Result<ExitStatus, blockchain::error::NodeError> {
    let node = blockchain::Node::new(config);
    node.start()
        .map(|mut process| process.wait().map_err(blockchain::error::NodeError::Io))
        .and_then(|status| status)
  }

  pub fn init_project(&self, path: PathBuf) -> Result<(), code_generator::error::ProjectGenerationError> {
    let generator = code_generator::CodeGenerator::new();
    generator.generate_project(path)
  }

  pub fn reset_project(&self, path: PathBuf) -> Result<(), code_generator::error::ProjectGenerationError> {
    let generator = code_generator::CodeGenerator::new();
    generator.reset_project(path)
  }
}
