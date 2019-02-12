pub mod blockchain;
pub mod code_generator;

use std::io;
use std::fs;
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

  pub fn start_node(&self, config: blockchain::NodeConfig) -> Result<ExitStatus, io::Error> {
    let node = blockchain::Node::new(config);
    node.start()
        .map(|mut process| process.wait())
        .and_then(|status| status)
  }

  pub fn init_project(&self, path: PathBuf) -> Result<(), io::Error> {
    let generator = code_generator::CodeGenerator::new();
    generator.generate_project(path)
  }

  pub fn reset_project(&self, path: PathBuf) -> Result<(), io::Error> {
    let vibranium_project_directory = path.join(code_generator::VIBRANIUM_PROJECT_DIRECTORY);

    if !vibranium_project_directory.exists() {
      return Err(io::Error::new(io::ErrorKind::NotFound, "Aborting. Not a Vibranium project."));
    }

    let _ = fs::remove_dir_all(vibranium_project_directory);
    let _ = fs::remove_dir_all(path.join(code_generator::DEFAULT_ARTIFACTS_DIRECTORY));
    Self::init_project(self, path)
  }
}
