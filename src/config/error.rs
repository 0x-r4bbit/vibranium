extern crate toml;
extern crate toml_query;

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
  Serialization(toml::ser::Error),
  Deserialization(toml::de::Error),
  Query(toml_query::error::Error),
  Io(io::Error),
  Other(String),
}

impl Error for ConfigError {
  fn description(&self) -> &str {
    match self {
      ConfigError::Serialization(error) => error.description(),
      ConfigError::Deserialization(error) => error.description(),
      ConfigError::Query(_error) => "",
      ConfigError::Io(error) => error.description(),
      ConfigError::Other(message) => message,
    }
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      ConfigError::Serialization(error) => Some(error),
      ConfigError::Deserialization(error) => Some(error),
      ConfigError::Query(_error) => None,
      ConfigError::Io(error) => Some(error),
      ConfigError::Other(_message) => None,
    }
  }
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConfigError::Serialization(error) => write!(f, "Couldn't serialize vibranium config: {}", error),
      ConfigError::Deserialization(error) => write!(f, "Couldn't deserialize vibranium config: {}", error),
      ConfigError::Query(error) => write!(f, "Couldn't query configuration: {}", error),
      ConfigError::Io(error) => write!(f, "Couldn't access configuration file: {}", error),
      ConfigError::Other(_message) => write!(f, "{}", self.description()),
    }
  }
}
