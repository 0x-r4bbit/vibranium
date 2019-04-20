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
