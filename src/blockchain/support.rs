use super::error;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

const PARITY_CLIENT_CMD: &str = "parity";
const GETH_CLIENT_CMD: &str = "geth";
const DEFAULT_DATADIR_NAME: &str = "datadir";
const DEFAULT_DATADIR_ENVIRONMENT: &str = "development";

pub enum SupportedBlockchainClients {
  Parity,
  Geth,
}

impl FromStr for SupportedBlockchainClients {
  type Err = error::NodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      PARITY_CLIENT_CMD => Ok(SupportedBlockchainClients::Parity),
      GETH_CLIENT_CMD => Ok(SupportedBlockchainClients::Geth),
      _ => Err(error::NodeError::UnsupportedClient),
    }
  }
}

impl ToString for SupportedBlockchainClients {
  fn to_string(&self) -> String {
    match self {
      SupportedBlockchainClients::Parity => PARITY_CLIENT_CMD.to_string(),
      SupportedBlockchainClients::Geth => GETH_CLIENT_CMD.to_string(),
    }
  }
}

pub fn default_options_from(client: SupportedBlockchainClients, vibranium_dir_path: &PathBuf) -> Vec<String> {
  match client {
    SupportedBlockchainClients::Parity => {
      vec![
        "--config".to_string(),
        "dev".to_string(),
        "--ws-origins".to_string(),
        "all".to_string(),
        "--base-path".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DATADIR_NAME)
          .join(DEFAULT_DATADIR_ENVIRONMENT)
          .to_string_lossy()
          .to_string(),
      ]
    },
    SupportedBlockchainClients::Geth => {
      vec![
        "--dev".to_string(),
        "--rpc".to_string(),
        "--ws".to_string(),
        "--wsorigins".to_string(),
        "*".to_string(),
        "--datadir".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DATADIR_NAME)
          .join(DEFAULT_DATADIR_ENVIRONMENT)
          .to_string_lossy()
          .to_string(),
      ]
    },
  }
}
