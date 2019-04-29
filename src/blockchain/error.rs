use std::io;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum NodeError {
  Io(io::Error),
  UnsupportedClient,
  Other(String),
}

impl Error for NodeError {
  fn description(&self) -> &str {
    match self {
      NodeError::Io(ref err) => err.description(),
      NodeError::UnsupportedClient => "No built-in support for request blockchain client. Please specify NodeConfig.client_options",
      NodeError::Other(message) => message,
    }
  }
}

impl fmt::Display for NodeError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      NodeError::Io(ref err) => {
        match err.kind() {
          io::ErrorKind::NotFound => write!(f, "Couldn't find executable for node client"),
          _ => write!(f, "{}", err),
        }
      },
      NodeError::UnsupportedClient => write!(f, "{}", self.description()),
      NodeError::Other(_message) => write!(f, "{}", self.description()),
    }
  }
}

#[derive(Debug)]
pub enum ConnectionError {
  UnsupportedProtocol,
  MissingConnectorConfig,
  Transport(web3::Error),
  Other(String),
}

impl Error for ConnectionError {
  fn description(&self) -> &str {
    match self {
      ConnectionError::UnsupportedProtocol => "Couldn't create blockchain connector. The configured protocol is not supported",
      ConnectionError::MissingConnectorConfig => "Couldn't find configuration for blockchain connector in project configuration.",
      ConnectionError::Transport(error) => error.description(),
      ConnectionError::Other(message) => message,
    }
  }
}

impl fmt::Display for ConnectionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConnectionError::UnsupportedProtocol => write!(f, "{}", self.description()),
      ConnectionError::MissingConnectorConfig => write!(f, "{}", self.description()),
      ConnectionError::Transport(_error) => write!(f, "{}", self.description()),
      ConnectionError::Other(_message) => write!(f, "{}", self.description()),
    }
  }
}
