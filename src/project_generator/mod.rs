extern crate toml;

use std::path::{PathBuf};
use std::fs;
use std::io::Write;

use crate::config;

pub mod error;

const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";
const DEFAULT_CONTRACTS_DIRECTORY: &str = "contracts";
const DEFAULT_ARTIFACTS_DIRECTORY: &str = "artifacts";

pub struct ProjectGenerator<'a> {
  config: &'a config::Config,
}

impl<'a> ProjectGenerator<'a> {
  pub fn new(config: &config::Config) -> ProjectGenerator {
    ProjectGenerator {
      config
    }
  }

  pub fn generate_project(&self, project_path: &PathBuf) -> Result<(), error::ProjectGenerationError> {
    if !project_path.exists() {
      return Err(error::ProjectGenerationError::ProjectPathNotFound);
    }

    let mut directories_to_create: Vec<String> = vec![VIBRANIUM_PROJECT_DIRECTORY.to_string(), DEFAULT_CONTRACTS_DIRECTORY.to_string()];

    if !self.config.exists() {
      directories_to_create.push(DEFAULT_ARTIFACTS_DIRECTORY.to_string());

      let config = config::ProjectConfig {
        artifacts_dir: DEFAULT_ARTIFACTS_DIRECTORY.to_string(),
        smart_contract_sources: vec![DEFAULT_CONTRACTS_DIRECTORY.to_string() + "/**"]
      };

      let config_toml = toml::to_string(&config).map_err(error::ProjectGenerationError::Serialization)?;
      let mut config_file = fs::File::create(&self.config.config_file).map_err(error::ProjectGenerationError::Io)?;
      config_file.write_all(config_toml.as_bytes()).map_err(error::ProjectGenerationError::Io)?;
    } else {
      let existing_config = self.config.read().map_err(error::ProjectGenerationError::InvalidConfig)?;
      directories_to_create.push(existing_config.artifacts_dir);
    }

    for directory in directories_to_create {
      let path = project_path.join(directory);
      if !path.exists() {
        fs::create_dir_all(path).map_err(error::ProjectGenerationError::Io)?;
      }
    }
    Ok(())
  }

  pub fn reset_project(&self, project_path: &PathBuf) -> Result<(), error::ProjectGenerationError> {
    let vibranium_project_directory = project_path.join(VIBRANIUM_PROJECT_DIRECTORY);

    if !vibranium_project_directory.exists() {
      return Err(error::ProjectGenerationError::VibraniumDirectoryNotFound);
    }

    if self.config.exists() {
      let existing_config = self.config.read().map_err(error::ProjectGenerationError::InvalidConfig)?;
      let _ = fs::remove_dir_all(project_path.join(existing_config.artifacts_dir));
    }

    let _ = fs::remove_dir_all(vibranium_project_directory);
    let _ = fs::remove_dir_all(project_path.join(DEFAULT_ARTIFACTS_DIRECTORY));

    Ok(())
  }
}
