extern crate toml;

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
  Serialization(toml::ser::Error),
  Deserialization(toml::de::Error),
  Io(io::Error),
}

impl Error for ConfigError {
  fn description(&self) -> &str {
    match self {
      ConfigError::Serialization(error) => error.description(),
      ConfigError::Deserialization(error) => error.description(),
      ConfigError::Io(error) => error.description(),
    }
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      ConfigError::Serialization(error) => Some(error),
      ConfigError::Deserialization(error) => Some(error),
      ConfigError::Io(error) => Some(error),
    }
  }
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConfigError::Serialization(error) => write!(f, "Couldn't serialize vibranium config: {}", error),
      ConfigError::Deserialization(error) => write!(f, "Couldn't deserialize vibranium config: {}", error),
      ConfigError::Io(error) => write!(f, "Couldn't access configuration file: {}", error),
    }
  }
}
