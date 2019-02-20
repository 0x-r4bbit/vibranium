pub mod blockchain;
pub mod project_generator;
pub mod config;

use std::process::ExitStatus;
use std::path::PathBuf;

#[macro_use]
extern crate serde_derive;

#[derive(Debug)]
pub struct Vibranium {
  project_path: PathBuf,
}

impl Vibranium {
  pub fn new(project_path: PathBuf) -> Vibranium {
    Vibranium {
      project_path
    }
  }

  pub fn start_node(&self, config: blockchain::NodeConfig) -> Result<ExitStatus, blockchain::error::NodeError> {
    let node = blockchain::Node::new(config);
    node.start()
        .map(|mut process| process.wait().map_err(blockchain::error::NodeError::Io))
        .and_then(|status| status)
  }

  pub fn init_project(&self) -> Result<(), project_generator::error::ProjectGenerationError> {
    let generator = project_generator::ProjectGenerator::new();
    generator.generate_project(&self.project_path)
  }

  pub fn reset_project(&self) -> Result<(), project_generator::error::ProjectGenerationError> {
    let generator = project_generator::ProjectGenerator::new();
    generator
      .reset_project(&self.project_path)
      .and_then(|_| generator.generate_project(&self.project_path))
  }
}
