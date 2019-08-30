use crate::blockchain::error::ConnectionError;
use std::convert::From;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AccountsError {
  Other(String),
}

impl Error for AccountsError {
  fn cause(&self) -> Option<&Error> {
    match self {
      AccountsError::Other(_message) => None,
    }
  }
}

impl fmt::Display for AccountsError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AccountsError::Other(message) => write!(f, "{}", message),
    }
  }
}

impl From<ConnectionError> for AccountsError {
  fn from(error: ConnectionError) -> Self {
    AccountsError::Other(error.to_string())
  }
}
