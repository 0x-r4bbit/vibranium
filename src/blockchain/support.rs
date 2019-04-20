use super::error;
use std::str::FromStr;
use std::string::ToString;

const DEFAULT_NODE_CLIENT: &str = "parity";

pub enum SupportedBlockchainClients {
  Parity,
}

impl FromStr for SupportedBlockchainClients {
  type Err = error::NodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      DEFAULT_NODE_CLIENT => Ok(SupportedBlockchainClients::Parity),
      _ => Err(error::NodeError::UnsupportedClient),
    }
  }
}

impl ToString for SupportedBlockchainClients {
  fn to_string(&self) -> String {
    match self {
      SupportedBlockchainClients::Parity => DEFAULT_NODE_CLIENT.to_string(),
    }
  }
}
