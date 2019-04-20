extern crate log;

use std::process::{Command, Child};
use crate::config;
use support::SupportedBlockchainClients;

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
        Some(config) => config.cmd.clone().unwrap_or_else(|| SupportedBlockchainClients::Parity.to_string()),
        None => SupportedBlockchainClients::Parity.to_string(),
      }
    });

    let client_options: Vec<String> = match &config.client_options {
      Some(options) => {
        match client.parse() {
          Ok(SupportedBlockchainClients::Parity) => merge_defaults_with_options_for(SupportedBlockchainClients::Parity, options.to_vec()),
          Ok(SupportedBlockchainClients::Geth) => merge_defaults_with_options_for(SupportedBlockchainClients::Geth, options.to_vec()),
          Err(_err) => options.to_vec(),
        }
      }
      None => {
        match project_config.blockchain {
          Some(config) => config.options.unwrap_or_else(|| {
            match client.parse() {
              Ok(SupportedBlockchainClients::Parity) => support::default_options_for(SupportedBlockchainClients::Parity),
              Ok(SupportedBlockchainClients::Geth) => support::default_options_for(SupportedBlockchainClients::Geth),
              Err(_err) => vec![],
            }
          }),
          None => support::default_options_for(SupportedBlockchainClients::Parity)
        }
      }
    };

    if client_options.is_empty() {
      if let Err(err) = client.parse::<SupportedBlockchainClients>() {
        Err(err)?
      }
    }

    info!("Starting node with command: {} {}", &client, client_options.join(" "));

    Command::new(client)
            .args(client_options)
            .spawn()
            .map_err(error::NodeError::Io)
  }
}

fn merge_defaults_with_options_for(client: SupportedBlockchainClients, options: Vec<String>) -> Vec<String> {
  let mut combined = support::default_options_for(client);
  combined.append(&mut options.clone());
  combined.sort();
  combined.dedup();
  combined
}
