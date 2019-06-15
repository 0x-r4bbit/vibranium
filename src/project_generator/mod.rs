extern crate toml;
extern crate log;

use std::path::{PathBuf};
use std::fs;
use std::io::Write;

use crate::config;

pub mod error;

pub const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";

pub struct ProjectGenerator<'a> {
  config: &'a config::Config,
}

pub struct ResetOptions {
  pub restore_config: bool,
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

    let mut directories_to_create: Vec<String> = vec![VIBRANIUM_PROJECT_DIRECTORY.to_string(), config::DEFAULT_CONTRACTS_DIRECTORY.to_string()];

    if !self.config.exists() {
      directories_to_create.push(config::DEFAULT_ARTIFACTS_DIRECTORY.to_string());
      self.create_default_config_file()?;
    } else {
      let existing_config = self.config.read()?;
      directories_to_create.push(existing_config.sources.artifacts);
    }

    for directory in directories_to_create {
      let path = project_path.join(directory);
      if !path.exists() {
        info!("Creating: {}", path.to_str().unwrap());
        fs::create_dir_all(path)?;
      }
    }
    Ok(())
  }

  pub fn reset_project(&self, project_path: &PathBuf, options: ResetOptions) -> Result<(), error::ProjectGenerationError> {
    self.check_vibranium_dir_exists()?;
    let vibranium_project_directory = project_path.join(VIBRANIUM_PROJECT_DIRECTORY);
    let default_artifacts_directory = project_path.join(config::DEFAULT_ARTIFACTS_DIRECTORY);

    if options.restore_config {
      info!("Restoring project's config file");
      self.create_default_config_file()?;
    }

    if self.config.exists() {
      let existing_config = self.config.read()?;
      if existing_config.sources.artifacts != config::DEFAULT_ARTIFACTS_DIRECTORY {
        let artifacts_dir = project_path.join(existing_config.sources.artifacts);
        info!("Removing: {}", &artifacts_dir.to_str().unwrap());
        let _ = fs::remove_dir_all(&artifacts_dir);
      }
    }

    info!("Removing: {}", &vibranium_project_directory.to_str().unwrap());
    let _ = fs::remove_dir_all(vibranium_project_directory);
    info!("Removing: {}", &default_artifacts_directory.to_str().unwrap());
    let _ = fs::remove_dir_all(&default_artifacts_directory);

    Ok(())
  }

  pub fn check_vibranium_dir_exists(&self) -> Result<(), error::ProjectGenerationError> {
    let vibranium_project_directory = self.config.project_path.join(VIBRANIUM_PROJECT_DIRECTORY);

    if !vibranium_project_directory.exists() {
      return Err(error::ProjectGenerationError::VibraniumDirectoryNotFound);
    }
    Ok(())
  }

  fn create_default_config_file(&self) -> Result<(), error::ProjectGenerationError> {
    let config = config::ProjectConfig::default();
    info!("Creating: {}", &self.config.config_file.to_str().unwrap());
    let config_toml = toml::to_string(&config)?;
    let mut config_file = fs::File::create(&self.config.config_file)?;
    config_file.write_all(config_toml.as_bytes())?;
    Ok(())
  }
}
