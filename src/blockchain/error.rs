use std::convert::From;
use std::io;
use std::fmt;
use std::error::Error;
use crate::config::error::ConfigError;

#[derive(Debug)]
pub enum NodeError {
  Io(io::Error),
  UnsupportedClient,
  Other(String),
}

impl Error for NodeError {
  fn cause(&self) -> Option<&Error> {
    match self {
      NodeError::Io(err) => Some(err),
      NodeError::UnsupportedClient => None,
      NodeError::Other(_message) => None,
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
      NodeError::UnsupportedClient => write!(f, "No built-in support for request blockchain client. Please specify NodeConfig.client_options"),
      NodeError::Other(message) => write!(f, "{}", message),
    }
  }
}

impl From<ConfigError> for NodeError {
  fn from(error: ConfigError) -> Self {
    NodeError::Other(error.to_string())
  }
}

impl From<io::Error> for NodeError {
  fn from(error: io::Error) -> Self {
    NodeError::Other(error.to_string())
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
  fn cause(&self) -> Option<&Error> {
    match self {
      ConnectionError::UnsupportedProtocol => None,
      ConnectionError::MissingConnectorConfig => None,
      ConnectionError::Transport(error) => Some(error),
      ConnectionError::Other(_message) => None,
    }
  }
}

impl fmt::Display for ConnectionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConnectionError::UnsupportedProtocol => write!(f, "Couldn't create blockchain connector. The configured protocol is not supported"),
      ConnectionError::MissingConnectorConfig => write!(f, "Couldn't find configuration for blockchain connector in project configuration."),
      ConnectionError::Transport(error) => write!(f, "{}", error),
      ConnectionError::Other(message) => write!(f, "{}", message),
    }
  }
}
