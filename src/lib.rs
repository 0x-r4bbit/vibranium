#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate glob;
extern crate web3;
extern crate ethabi;
extern crate petgraph;
extern crate regex;
extern crate sha3;
extern crate toml;
extern crate toml_query;

pub mod blockchain;
pub mod project_generator;
pub mod compiler;
pub mod config;
pub mod deployment;
mod utils;

use std::process::{ExitStatus, Output};
use std::path::PathBuf;
use std::collections::HashMap;
use blockchain::connector as connector;
use web3::types::Address;
use project_generator::error::ProjectGenerationError;
use utils::adjust_canonicalization;

#[derive(Debug)]
pub struct Vibranium {
  project_path: PathBuf,
  pub config: config::Config,
}

impl Vibranium {
  pub fn new(project_path: PathBuf) -> Result<Vibranium, ProjectGenerationError> {
    let mut project_path = project_path.canonicalize().map_err(|_err| ProjectGenerationError::ProjectPathNotFound)?;
    project_path = adjust_canonicalization(&project_path);
    Ok(Vibranium {
      config: config::Config::new(project_path.clone()),
      project_path,
    })
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
      .and_then(|_| compiler.compile(config).map(|process| {
        process.wait_with_output().map_err(compiler::error::CompilerError::Io)
      }))
      .and_then(|output| output)
      .and_then(|output| {
        if !output.status.success() {
          Err(compiler::error::CompilerError::Other(String::from_utf8_lossy(&output.stderr).to_string()))
        } else {
          Ok(output)
        }
      })
  }

  pub fn get_blockchain_connector(&self) -> Result<(web3::transports::EventLoopHandle, connector::BlockchainConnector), blockchain::error::ConnectionError> {
    let generator = project_generator::ProjectGenerator::new(&self.config);

    generator
      .check_vibranium_dir_exists()
      .map_err(|err| blockchain::error::ConnectionError::Other(err.to_string()))
      .and_then(|_| {
        let project_config = self.config.read().map_err(|err| blockchain::error::ConnectionError::Other(err.to_string()))?;
        let blockchain_config = project_config.blockchain.ok_or(blockchain::error::ConnectionError::MissingConnectorConfig)?;
        let connector_config = blockchain_config.connector.ok_or(blockchain::error::ConnectionError::MissingConnectorConfig)?;
        let (eloop, adapter) = connector::web3_adapter::Web3Adapter::new(connector_config)?;
        let blockchain_connector = connector::BlockchainConnector::new(adapter);
        Ok((eloop, blockchain_connector))
      })
  }

  pub fn deploy(&self, options: deployment::DeployOptions) -> Result<HashMap<Address, (String, Address, String, bool)>, deployment::error::DeploymentError> {
    let (_eloop, connector) = self.get_blockchain_connector().map_err(deployment::error::DeploymentError::Connection)?;
    let tracker = deployment::tracker::DeploymentTracker::new(&self.config);
    let deployer = deployment::Deployer::new(&self.config, &connector, &tracker);
    deployer.deploy(options)
  }

  pub fn get_tracking_data(&self) -> Result<Option<deployment::tracker::SmartContractTrackingData>, deployment::error::DeploymentTrackingError> {
    let (_eloop, connector) = self.get_blockchain_connector()?;
    let tracker = deployment::tracker::DeploymentTracker::new(&self.config);
    connector.get_first_block()
      .map_err(|err| deployment::error::DeploymentTrackingError::Other(err.to_string()))
      .and_then(|block| tracker.get_all_smart_contract_tracking_data(&block.unwrap().hash.unwrap()))
  }
}
