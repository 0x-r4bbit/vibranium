use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use toml;
use toml_query;

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

impl From<io::Error> for ConfigError {
  fn from(error: io::Error) -> Self {
    ConfigError::Io(error)
  }
}

impl From<toml::de::Error> for ConfigError {
  fn from(error: toml::de::Error) -> Self {
    ConfigError::Deserialization(error)
  }
}

impl From<toml::ser::Error> for ConfigError {
  fn from(error: toml::ser::Error) -> Self {
    ConfigError::Serialization(error)
  }
}
