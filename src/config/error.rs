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
  Deletion(toml_query::error::Error),
  Io(io::Error),
  Other(String),
}

impl Error for ConfigError {
  fn cause(&self) -> Option<&Error> {
    match self {
      ConfigError::Serialization(error) => Some(error),
      ConfigError::Deserialization(error) => Some(error),
      ConfigError::Query(_error) => None,
      ConfigError::Deletion(_error) => None,
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
      ConfigError::Deletion(error) => write!(f, "{}", error),
      ConfigError::Io(error) => write!(f, "Couldn't access configuration file: {}", error),
      ConfigError::Other(message) => write!(f, "{}", message),
    }
  }
}
