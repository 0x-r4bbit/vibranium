extern crate toml;

use std::path::{PathBuf};
use std::fs;
use std::io::Write;

const VIBRANIUM_CONFIG_FILE: &str = "vibranium.toml";
pub const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";
const DEFAULT_CONTRACTS_DIRECTORY: &str = "contracts";
pub const DEFAULT_ARTIFACTS_DIRECTORY: &str = "artifacts";

#[derive(Serialize)]
struct ProjectConfig {
  artifacts_dir: String,
  smart_contract_sources: Vec<String>,
}

pub struct CodeGenerator;

impl CodeGenerator {
  pub fn new() -> CodeGenerator {
    CodeGenerator
  }

  pub fn generate_project(&self, project_path: PathBuf) -> Result<(), std::io::Error> {
    for directory in vec![VIBRANIUM_PROJECT_DIRECTORY, DEFAULT_CONTRACTS_DIRECTORY] {
      let path = project_path.join(directory);
      if !path.exists() {
        fs::create_dir_all(path)?;
      }
    }

    let config_path = project_path.join(VIBRANIUM_CONFIG_FILE);

    if !config_path.exists() {
      let config = ProjectConfig {
        artifacts_dir: DEFAULT_ARTIFACTS_DIRECTORY.to_string(),
        smart_contract_sources: vec![DEFAULT_CONTRACTS_DIRECTORY.to_string() + "/**"]
      };

      let config_toml = toml::to_string(&config).unwrap();
      let mut config_file = fs::File::create(config_path)?;
      config_file.write_all(config_toml.as_bytes())?;
    }
    Ok(())
  }
}
