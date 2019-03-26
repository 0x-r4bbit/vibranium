pub mod error;

use std::fs;
use std::path::PathBuf;

pub const VIBRANIUM_CONFIG_FILE: &str = "vibranium.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig {
  pub artifacts_dir: String,
  pub smart_contract_sources: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Config {
  pub project_path: PathBuf,
  pub config_file: PathBuf,
}

impl Config {
  pub fn new(path: PathBuf) -> Config {
    Config {
      project_path: path.clone(),
      config_file: path.join(VIBRANIUM_CONFIG_FILE)
    }
  }

  pub fn exists(&self) -> bool {
    self.config_file.exists()
  }

  pub fn read(&self) -> Result<ProjectConfig, error::ConfigError> {
    toml::from_str(&fs::read_to_string(&self.config_file).map_err(error::ConfigError::Io)?).map_err(error::ConfigError::Deserialization)
  }
}

