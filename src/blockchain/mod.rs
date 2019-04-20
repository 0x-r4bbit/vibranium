extern crate log;

use std::process::{Command, Child};
use crate::config;

pub mod error;
pub mod support;

pub struct NodeConfig {
  pub client: Option<String>,
  pub client_options: Option<Vec<String>>,
}

pub struct Node<'a> {
  config: &'a config::Config
}

impl<'a> Node<'a> {
  pub fn new(config: &config::Config) -> Node {
    Node {
      config,
    }
  }

  pub fn start(&self, config: NodeConfig) -> Result<Child, error::NodeError> {
    let project_config = self.config.read().map_err(|error| error::NodeError::Other(error.to_string()))?;

    let client = config.client.unwrap_or_else(|| {
      match &project_config.blockchain {
        Some(config) => config.cmd.clone().unwrap_or_else(|| support::SupportedBlockchainClients::Parity.to_string()),
        None => support::SupportedBlockchainClients::Parity.to_string(),
      }
    });

    let client_options: Vec<String> = match &config.client_options {
      Some(options) => options.to_vec(),
      None => {
        match project_config.blockchain {
          Some(config) => config.options.unwrap_or(default_options()),
          None => default_options()
        }
      }
    };

    info!("Starting node with command: {} {}", &client, client_options.join(" "));

    Command::new(client)
            .args(client_options)
            .spawn()
            .map_err(error::NodeError::Io)
  }
}

pub fn default_options() -> Vec<String> {
  vec!["--config".to_string(), "dev".to_string()]
}
