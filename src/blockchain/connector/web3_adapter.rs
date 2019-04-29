use super::{SupportedProtocols, BlockchainConnectorConfig};
use super::super::error::ConnectionError;
use super::super::super::utils;
use web3::futures::Future;
use web3::helpers::CallFuture;
use jsonrpc_core as rpc;

#[derive(Debug, Clone)]
enum Transports {
  Http(web3::transports::Http),
  Ws(web3::transports::WebSocket),
}

impl web3::Transport for Transports {
  type Out = Box<dyn Future<Item = rpc::Value, Error = web3::Error>>;
  fn prepare(&self, method: &str, params: Vec<rpc::Value>) -> (web3::RequestId, rpc::Call) {
    match self {
      Transports::Http(transport) => transport.prepare(&method, params),
      Transports::Ws(transport) => transport.prepare(&method, params)
    }
  }

  fn send(&self, id: web3::RequestId, request: rpc::Call) -> Self::Out {
    match self {
      Transports::Http(transport) => Box::new(transport.send(id, request)),
      Transports::Ws(transport) => Box::new(transport.send(id, request))
    }
  }
}

pub struct Web3Adapter {
  web3: web3::Web3<Transports>
}

impl Web3Adapter {
  pub fn new(config: BlockchainConnectorConfig) -> Result<(web3::transports::EventLoopHandle, Web3Adapter), ConnectionError> {
    let (eloop, transport) = match config.protocol.parse() {
      Ok(SupportedProtocols::Rpc) => {
        let (eloop, transport) = web3::transports::Http::new(&format!("http://{}:{}", utils::normalize_localhost(config.host), config.port)).unwrap();
        (eloop, Transports::Http(transport))
      },
      Ok(SupportedProtocols::Ws) => {
        let (eloop, transport) = web3::transports::WebSocket::new(&format!("ws://{}:{}", utils::normalize_localhost(config.host), config.port)).unwrap();
        (eloop, Transports::Ws(transport))
      },
      Err(err) => Err(err)?,
    };

    let web3 = web3::Web3::new(transport);

    Ok((eloop, Web3Adapter { web3 }))
  }

  pub fn accounts(&self) -> CallFuture<Vec<web3::types::Address>, Box<dyn Future<Item = rpc::Value, Error = web3::Error>>> {
    self.web3.eth().accounts()
  }
}
