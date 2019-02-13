use std::io;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum NodeError {
  Io(io::Error),
}

impl fmt::Display for NodeError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      NodeError::Io(ref err) => {
        match err.kind() {
          io::ErrorKind::NotFound => write!(f, "Couldn't find executable for node client"),
          _ => write!(f, "{}", err),
        }
      }
    }
  }
}

impl error::Error for NodeError {
  fn description(&self) -> &str {
    match *self {
      NodeError::Io(ref err) => err.description(),
    }
  }
}


