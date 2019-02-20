pub mod error;

use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig {
  pub artifacts_dir: String,
  pub smart_contract_sources: Vec<String>,
}

pub fn read(path: &PathBuf) -> Result<ProjectConfig, error::ConfigError> {
  toml::from_str(&fs::read_to_string(path).map_err(error::ConfigError::Io)?).map_err(error::ConfigError::Deserialization)
}

