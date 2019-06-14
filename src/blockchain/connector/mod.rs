pub mod web3_adapter;

use std::str::FromStr;
use std::string::ToString;
use super::error::ConnectionError;
use web3_adapter::Web3Adapter;
use web3::futures::Future;
use web3::types::{Address, Block, BlockId, BlockNumber, H256, U256};
use jsonrpc_core as rpc;


pub type CallFuture = web3::helpers::CallFuture<Vec<Address>, Box<dyn Future<Item = rpc::Value, Error = web3::Error>>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainConnectorConfig {
  pub protocol: String,
  pub host: String,
  pub port: String,
}

impl Default for BlockchainConnectorConfig {
  fn default() -> Self {
    BlockchainConnectorConfig {
      protocol: SupportedProtocols::Rpc.to_string(),
      host: "localhost".to_string(),
      port: "8545".to_string(),
    }
  }
}

pub enum SupportedProtocols {
  Rpc,
  Ws,
}

impl FromStr for SupportedProtocols {
  type Err = ConnectionError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "rpc" => Ok(SupportedProtocols::Rpc),
      "ws" => Ok(SupportedProtocols::Ws),
      _ => Err(ConnectionError::UnsupportedProtocol),
    }
  }
}

impl ToString for SupportedProtocols {
  fn to_string(&self) -> String {
    match self {
      SupportedProtocols::Rpc => "rpc".to_string(),
      SupportedProtocols::Ws => "ws".to_string(),
    }
  }
}

pub struct BlockchainConnector {
  adapter: Web3Adapter,
}

impl BlockchainConnector {
  pub fn new(adapter: Web3Adapter) -> BlockchainConnector {
    BlockchainConnector {
      adapter,
    }
  }

  pub fn accounts(&self) -> Result<Vec<Address>, ConnectionError> {
    self.adapter.accounts().wait().map_err(ConnectionError::Transport)
  }

  pub fn balance(&self, address: Address, block_number: Option<BlockNumber>) -> Result<U256, ConnectionError> {
    self.adapter.balance(address, block_number).wait().map_err(ConnectionError::Transport)
  }

  pub fn gas_price(&self) -> Result<U256, ConnectionError> {
    self.adapter.gas_price().wait().map_err(ConnectionError::Transport)
  }

  pub fn get_block(&self, block: BlockId) -> Result<Option<Block<H256>>, ConnectionError> {
    self.adapter.get_block(block).wait().map_err(ConnectionError::Transport)
  }

  pub fn deploy(&self, bytes: &[u8]) -> Result<web3::contract::deploy::Builder<web3_adapter::Transports>, ethabi::Error> {
    self.adapter.deploy(bytes)
  }
}
