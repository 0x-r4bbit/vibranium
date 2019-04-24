pub mod blockchain;
pub mod project_generator;
pub mod compiler;
pub mod config;
mod utils;

use std::process::{ExitStatus, Output};
use std::path::PathBuf;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate glob;

#[derive(Debug)]
pub struct Vibranium {
  project_path: PathBuf,
  pub config: config::Config,
}

impl Vibranium {
  pub fn new(project_path: PathBuf) -> Vibranium {
    Vibranium {
      config: config::Config::new(project_path.clone()),
      project_path,
    }
  }

  pub fn start_node(&self, config: blockchain::NodeConfig) -> Result<ExitStatus, blockchain::error::NodeError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);
    generator
      .check_vibranium_dir_exists()
      .map_err(|error| blockchain::error::NodeError::Other(error.to_string()))
      .and_then(|_| {
        let node = blockchain::Node::new(&self.config);
        node.start(config)
          .map(|mut process| process.wait().map_err(blockchain::error::NodeError::Io))
          .and_then(|status| status)
      })
  }

  pub fn init_project(&self) -> Result<(), project_generator::error::ProjectGenerationError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);
    generator.generate_project(&self.project_path)
  }

  pub fn reset_project(&self, reset_options: project_generator::ResetOptions) -> Result<(), project_generator::error::ProjectGenerationError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);
    generator
      .reset_project(&self.project_path, reset_options)
      .and_then(|_| generator.generate_project(&self.project_path))
  }

  pub fn set_config(&self, option: String, value: toml::Value) -> Result<(), config::error::ConfigError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);
    generator
      .check_vibranium_dir_exists()
      .map_err(|error| config::error::ConfigError::Other(error.to_string()))
      .and_then(|_| self.config.write(option, value))
  }

  pub fn unset_config(&self, option: String) -> Result<(), config::error::ConfigError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);
    generator
      .check_vibranium_dir_exists()
      .map_err(|error| config::error::ConfigError::Other(error.to_string()))
      .and_then(|_| self.config.remove(option))
  }

  pub fn compile(&self, config: compiler::CompilerConfig) -> Result<Output, compiler::error::CompilerError> {
    let compiler = compiler::Compiler::new(&self.config);
    let generator = project_generator::ProjectGenerator::new(&self.config);

    generator
      .check_vibranium_dir_exists()
      .map_err(compiler::error::CompilerError::VibraniumDirectoryNotFound)
      .and_then(|_| {
        compiler.compile(config).map(|process| {
          process.wait_with_output().map_err(compiler::error::CompilerError::Io)
        })
      })
      .and_then(|output| output)
      .and_then(|output| {
        if !output.stderr.is_empty() {
          Err(compiler::error::CompilerError::Other(String::from_utf8_lossy(&output.stderr).to_string()))
        } else {
          Ok(output)
        }
      })
  }
}
