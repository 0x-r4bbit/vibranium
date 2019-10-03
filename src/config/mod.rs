pub mod error;

use crate::blockchain;
use crate::compiler;
use crate::project_generator;

use blockchain::connector::BlockchainConnectorConfig;
use project_generator::VIBRANIUM_PROJECT_DIRECTORY;
use std::default::Default;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use toml;
use toml_query::delete::TomlValueDeleteExt;
use toml_query::error::Error::IdentifierNotFoundInDocument;
use toml_query::insert::TomlValueInsertExt;
use toml_query::set::TomlValueSetExt;

pub const VIBRANIUM_CONFIG_FILE: &str = "vibranium.toml";
pub const DEFAULT_ARTIFACTS_DIRECTORY: &str = "artifacts";
pub const DEFAULT_CONTRACTS_DIRECTORY: &str = "contracts";

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfig {
    pub sources: ProjectSourcesConfig,
    pub compiler: Option<ProjectCmdExecutionConfig>,
    pub blockchain: Option<ProjectBlockchainConfig>,
    pub deployment: Option<ProjectDeploymentConfig>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        ProjectConfig {
            sources: ProjectSourcesConfig::default(),
            compiler: Some(ProjectCmdExecutionConfig::default()),
            blockchain: Some(ProjectBlockchainConfig::default()),
            deployment: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectCmdExecutionConfig {
    pub cmd: Option<String>,
    pub options: Option<Vec<String>>,
}

impl Default for ProjectCmdExecutionConfig {
    fn default() -> Self {
        ProjectCmdExecutionConfig {
            cmd: Some(compiler::support::SupportedCompilers::Solc.to_string()),
            options: Some(compiler::support::default_options_from(
                compiler::support::SupportedCompilers::Solc,
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectBlockchainConfig {
    pub cmd: Option<String>,
    pub options: Option<Vec<String>>,
    pub connector: Option<BlockchainConnectorConfig>,
}

impl Default for ProjectBlockchainConfig {
    fn default() -> Self {
        ProjectBlockchainConfig {
            cmd: Some(blockchain::support::SupportedBlockchainClients::Parity.to_string()),
            options: None,
            connector: Some(blockchain::connector::BlockchainConnectorConfig::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSourcesConfig {
    pub artifacts: String,
    pub smart_contracts: Vec<String>,
}

impl Default for ProjectSourcesConfig {
    fn default() -> Self {
        ProjectSourcesConfig {
            artifacts: DEFAULT_ARTIFACTS_DIRECTORY.to_string(),
            smart_contracts: vec![DEFAULT_CONTRACTS_DIRECTORY.to_string() + "/*.sol"],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDeploymentConfig {
    pub tx_confirmations: Option<usize>,
    pub gas_price: Option<usize>,
    pub gas_limit: Option<usize>,
    pub tracking_enabled: Option<bool>,
    pub smart_contracts: Vec<SmartContractConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SmartContractConfig {
    pub name: String,
    pub address: Option<String>,
    pub args: Option<Vec<SmartContractArg>>,
    pub gas_price: Option<usize>,
    pub gas_limit: Option<usize>,
    pub instance_of: Option<String>,
    pub abi_path: Option<String>,
    pub bytecode_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SmartContractArg {
    pub value: String,
    pub kind: String,
}

#[derive(Default, Debug)]
pub struct Config {
    pub project_path: PathBuf,
    pub vibranium_dir_path: PathBuf,
    pub config_file: PathBuf,
}

impl Config {
    pub fn new(path: PathBuf) -> Config {
        Config {
            project_path: path.clone(),
            vibranium_dir_path: path.clone().join(VIBRANIUM_PROJECT_DIRECTORY),
            config_file: path.join(VIBRANIUM_CONFIG_FILE),
        }
    }

    pub fn exists(&self) -> bool {
        self.config_file.exists()
    }

    pub fn read(&self) -> Result<ProjectConfig, error::ConfigError> {
        toml::from_str(&fs::read_to_string(&self.config_file)?)
            .map_err(error::ConfigError::Deserialization)
    }

    pub fn write(&self, option: String, value: toml::Value) -> Result<(), error::ConfigError> {
        let mut config = self.try_from_config_file()?;

        if let Err(err) = config.set(&option, value.clone()) {
            match err {
                IdentifierNotFoundInDocument(_message) => {
                    config
                        .insert(&option, value.clone())
                        .map_err(error::ConfigError::Query)?;
                }
                _ => Err(error::ConfigError::Query(err))?,
            }
        }

        config
            .try_into::<ProjectConfig>()
            .map_err(error::ConfigError::Deserialization)
            .and_then(|cfg| {
                let config_toml = toml::to_string(&cfg)?;
                let mut config_file = fs::File::create(&self.config_file)?;
                config_file
                    .write_all(config_toml.as_bytes())
                    .map_err(error::ConfigError::Io)
            })?;

        Ok(())
    }

    pub fn remove(&self, option: String) -> Result<(), error::ConfigError> {
        let mut config = self.try_from_config_file()?;

        if let Err(err) = config.delete(&option) {
            match err {
                IdentifierNotFoundInDocument(field) => {
                    info!("Couldn't delete unsupported option {}", field);
                }
                _ => Err(error::ConfigError::Deletion(err))?,
            }
        }

        self.try_into_config_file(config.clone())?;
        Ok(())
    }

    fn try_from_config_file(&self) -> Result<toml::Value, error::ConfigError> {
        toml::Value::try_from(self.read()?).map_err(error::ConfigError::Serialization)
    }

    fn try_into_config_file(&self, config: toml::Value) -> Result<(), error::ConfigError> {
        config
            .try_into::<ProjectConfig>()
            .map_err(error::ConfigError::Deserialization)
            .and_then(|cfg| {
                let config_toml = toml::to_string(&cfg)?;
                let mut config_file = fs::File::create(&self.config_file)?;
                config_file
                    .write_all(config_toml.as_bytes())
                    .map_err(error::ConfigError::Io)
            })
    }
}
