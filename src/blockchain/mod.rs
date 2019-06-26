use std::process::{Command, Child};
use std::path::PathBuf;

use crate::config;
use crate::utils;

use support::SupportedBlockchainClients;

pub mod error;
pub mod support;
pub mod connector;

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
    let project_config = self.config.read()?;

    let client = config.client.unwrap_or_else(|| {
      match &project_config.blockchain {
        Some(config) => config.cmd.clone().unwrap_or_else(|| SupportedBlockchainClients::Parity.to_string()),
        None => SupportedBlockchainClients::Parity.to_string(),
      }
    });

    let client_options: Vec<String> = match &config.client_options {
      Some(options) => {
        match client.parse() {
          Ok(SupportedBlockchainClients::Parity) => utils::merge_cli_options(
            support::default_options_from(SupportedBlockchainClients::Parity, &self.config.vibranium_dir_path),
            options.to_vec()
          ),
          Ok(SupportedBlockchainClients::Geth) => utils::merge_cli_options(
            support::default_options_from(SupportedBlockchainClients::Geth, &self.config.vibranium_dir_path),
            options.to_vec()
          ),
          Err(_err) => options.to_vec(),
        }
      }
      None => {
        match project_config.blockchain {
          Some(config) => config.options.unwrap_or_else(|| try_default_options_from(&client, &self.config.vibranium_dir_path)),
          None => try_default_options_from(&client, &self.config.vibranium_dir_path)
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


fn try_default_options_from(client: &str, vibranium_dir_path: &PathBuf) -> Vec<String> {
  match client.parse() {
    Ok(SupportedBlockchainClients::Parity) => support::default_options_from(SupportedBlockchainClients::Parity, vibranium_dir_path),
    Ok(SupportedBlockchainClients::Geth) => support::default_options_from(SupportedBlockchainClients::Geth, vibranium_dir_path),
    Err(_err) => vec![],
  }
}
