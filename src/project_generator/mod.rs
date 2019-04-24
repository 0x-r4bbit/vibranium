extern crate toml;
extern crate log;

use std::path::{PathBuf};
use std::fs;
use std::io::Write;

use crate::blockchain;
use crate::compiler;
use crate::config;

pub mod error;

const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";
const DEFAULT_CONTRACTS_DIRECTORY: &str = "contracts";
const DEFAULT_ARTIFACTS_DIRECTORY: &str = "artifacts";

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

    let mut directories_to_create: Vec<String> = vec![VIBRANIUM_PROJECT_DIRECTORY.to_string(), DEFAULT_CONTRACTS_DIRECTORY.to_string()];

    if !self.config.exists() {
      directories_to_create.push(DEFAULT_ARTIFACTS_DIRECTORY.to_string());
      self.create_default_config_file()?;
    } else {
      let existing_config = self.config.read().map_err(error::ProjectGenerationError::InvalidConfig)?;
      directories_to_create.push(existing_config.sources.artifacts);
    }

    for directory in directories_to_create {
      let path = project_path.join(directory);
      if !path.exists() {
        info!("Creating: {}", path.to_str().unwrap());
        fs::create_dir_all(path).map_err(error::ProjectGenerationError::Io)?;
      }
    }
    Ok(())
  }

  pub fn reset_project(&self, project_path: &PathBuf, options: ResetOptions) -> Result<(), error::ProjectGenerationError> {
    self.check_vibranium_dir_exists()?;
    let vibranium_project_directory = project_path.join(VIBRANIUM_PROJECT_DIRECTORY);
    let default_artifacts_directory = project_path.join(DEFAULT_ARTIFACTS_DIRECTORY);

    if options.restore_config {
      info!("Restoring project's config file");
      self.create_default_config_file()?;
    }

    if self.config.exists() {
      let existing_config = self.config.read().map_err(error::ProjectGenerationError::InvalidConfig)?;
      if existing_config.sources.artifacts != DEFAULT_ARTIFACTS_DIRECTORY {
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
    let config = default_project_config();
    info!("Creating: {}", &self.config.config_file.to_str().unwrap());
    let config_toml = toml::to_string(&config).map_err(error::ProjectGenerationError::Serialization)?;
    let mut config_file = fs::File::create(&self.config.config_file).map_err(error::ProjectGenerationError::Io)?;
    config_file.write_all(config_toml.as_bytes()).map_err(error::ProjectGenerationError::Io)?;
    Ok(())
  }
}

pub fn default_project_config() -> config::ProjectConfig {
  config::ProjectConfig {
    sources: config::ProjectSourcesConfig {
      artifacts: DEFAULT_ARTIFACTS_DIRECTORY.to_string(),
      smart_contracts: vec![DEFAULT_CONTRACTS_DIRECTORY.to_string() + "/*.sol"],
    },
    compiler: Some(config::ProjectCmdExecutionConfig {
      cmd: Some(compiler::support::SupportedCompilers::Solc.to_string()),
      options: Some(compiler::strategy::solc::default_options())
    }),
    blockchain: Some(config::ProjectCmdExecutionConfig {
      cmd: Some(blockchain::support::SupportedBlockchainClients::Parity.to_string()),
      options: Some(blockchain::support::default_options_from(blockchain::support::SupportedBlockchainClients::Parity))
    }),
  }
}
