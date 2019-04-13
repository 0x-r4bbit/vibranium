extern crate toml;
extern crate toml_query;
pub mod error;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use toml_query::set::TomlValueSetExt;
use toml_query::insert::TomlValueInsertExt;
use toml_query::error::Error::IdentifierNotFoundInDocument;

pub const VIBRANIUM_CONFIG_FILE: &str = "vibranium.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig {
  pub sources: ProjectSourcesConfig,
  pub compiler: Option<ProjectCompilerConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectCompilerConfig {
  pub cmd: Option<String>,
  pub options: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSourcesConfig {
  pub artifacts: String,
  pub smart_contracts: Vec<String>,
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

  pub fn write(&self, option: String, value: toml::Value) -> Result<(), error::ConfigError> {
    let mut config = toml::Value::try_from(self.read()?).map_err(error::ConfigError::Serialization)?;

    if let Err(err) = config.set(&option, value.clone()) {
      match err {
        IdentifierNotFoundInDocument(_message) => {
          config.insert(&option, value.clone()).map_err(error::ConfigError::Query)?;
        },
        _ => Err(error::ConfigError::Query(err))?
      }
    }

    config.try_into::<ProjectConfig>()
      .map_err(error::ConfigError::Deserialization)
      .and_then(|cfg| {
        let config_toml = toml::to_string(&cfg).map_err(error::ConfigError::Serialization)?;
        let mut config_file = fs::File::create(&self.config_file)
          .map_err(error::ConfigError::Io)?;

        config_file.write_all(config_toml.as_bytes()).map_err(error::ConfigError::Io)
      })?;

    Ok(())
  }
}

