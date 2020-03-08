use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use toml;
use toml_query;
use ethabi;

use crate::blockchain;
use crate::config;

#[derive(Debug)]
pub enum DeploymentError {
  MissingConfig,
  InvalidParamType(ethabi::Error),
  TokenizeParam(ethabi::Error, String),
  NothingToDeploy,
  CyclicDependency(String),
  MissingArtifact(String, String),
  MissingABIPath(String),
  MissingBytecodePath(String),
  TooManyConstructorArgs(String),
  MissingConfigForReference(String),
  InvalidAddress(String, String),
  Connection(blockchain::error::ConnectionError),
  DeployContract(web3::contract::deploy::Error, String),
  InvalidConstructorArgs(ethabi::Error, String),
  TrackingError(DeploymentTrackingError),
  Other(String),
}

impl Error for DeploymentError {
  fn cause(&self) -> Option<&dyn Error> {
    match self {
      DeploymentError::MissingConfig => None,
      DeploymentError::InvalidParamType(error) => Some(error),
      DeploymentError::TokenizeParam(error, _value) => Some(error),
      DeploymentError::NothingToDeploy => None,
      DeploymentError::CyclicDependency(_name) => None,
      DeploymentError::MissingArtifact(_kind, _name) => None,
      DeploymentError::MissingABIPath(_name) => None,
      DeploymentError::MissingBytecodePath(_name) => None,
      DeploymentError::TooManyConstructorArgs(_name) => None,
      DeploymentError::MissingConfigForReference(_reference) => None,
      DeploymentError::InvalidAddress(_name, _message) => None,
      DeploymentError::Connection(error) => Some(error),
      DeploymentError::DeployContract(error, _name) => Some(error),
      DeploymentError::InvalidConstructorArgs(error, _name) => Some(error),
      DeploymentError::TrackingError(error) => Some(error),
      DeploymentError::Other(_message) => None,
    }
  } 
}

impl fmt::Display for DeploymentError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DeploymentError::MissingConfig => write!(f, "Missing deployment configuration."),
      DeploymentError::InvalidParamType(error) => write!(f, "Couldn't read Smart Contract constructor parameter: {}", error),
      DeploymentError::TokenizeParam(error, value) => write!(f, "Couldn't tokenize Smart Contract constructor parameter: {} with value {:?}", error, value),
      DeploymentError::NothingToDeploy => write!(f, "Couldn't find artifacts to deploy. Please compile first."),
      DeploymentError::CyclicDependency(name) => write!(f, "Couldn't deploy Smart Contracts due to a cyclic dependency in '{}'", name),
      DeploymentError::MissingArtifact(kind, name) => write!(f, "Couldn't find {} file for artifact '{}'", kind, name),
      DeploymentError::MissingABIPath(name) => write!(f, "Missing `abi_path` for Smart Contract configuration '{}'", name),
      DeploymentError::MissingBytecodePath(name) => write!(f, "Missing `bytecode_path` for Smart Contract configuration '{}'", name),
      DeploymentError::TooManyConstructorArgs(name) => write!(f, "Couldn't deploy Smart Contract '{}' due to too many constructor arguments (max. 10)", name),
      DeploymentError::MissingConfigForReference(reference) => write!(f, "Couldn't find Smart Contract configuration for reference '{}'", reference),
      DeploymentError::InvalidAddress(name, message) => write!(f, "Invalid address in Smart Contract configuration for '{}': {}", name, message),
      DeploymentError::Connection(error) => write!(f, "{}", error),
      DeploymentError::DeployContract(error, name) => write!(f, "Couldn't deploy Smart Contract '{}' due to {}", name, error),
      DeploymentError::InvalidConstructorArgs(_error, name) => write!(f, "Couldn't deploy Smart Contract '{}' due to mismatching types in constructor arguments.", name),
      DeploymentError::TrackingError(error) => write!(f, "Couldn't track deployed Smart Contracts: {}", error),
      DeploymentError::Other(message) => write!(f, "{}", message),
    }
  }
}

impl From<io::Error> for DeploymentError {
  fn from(error: io::Error) -> Self {
    DeploymentError::Other(error.to_string())
  }
}

impl From<config::error::ConfigError> for DeploymentError {
  fn from(error: config::error::ConfigError) -> Self {
    DeploymentError::Other(error.to_string())
  }
}

impl From<blockchain::error::ConnectionError> for DeploymentError {
  fn from(error: blockchain::error::ConnectionError) -> Self {
    DeploymentError::Connection(error)
  }
}

impl From<DeploymentTrackingError> for DeploymentError {
  fn from(error: DeploymentTrackingError) -> Self {
    DeploymentError::TrackingError(error)
  }
}

impl From<ethabi::Error> for DeploymentError {
  fn from(error: ethabi::Error) -> Self {
    DeploymentError::Other(error.to_string())
  }
}

#[derive(Debug)]
pub enum DeploymentTrackingError {
  Other(String),
  DatabaseNotFound,
  Deserialization(toml::de::Error),
  Serialization(toml::ser::Error),
  Insertion(toml_query::error::Error),
  Read(toml_query::error::Error),
  Set(toml_query::error::Error),
  Delete(toml_query::error::Error),
}

impl Error for DeploymentTrackingError {
  fn cause(&self) -> Option<&dyn Error> {
    match self {
      DeploymentTrackingError::Other(_) => None,
      DeploymentTrackingError::DatabaseNotFound => None,
      DeploymentTrackingError::Deserialization(error) => Some(error),
      DeploymentTrackingError::Serialization(error) => Some(error),
      DeploymentTrackingError::Insertion(_error) => None,
      DeploymentTrackingError::Read(_error) => None,
      DeploymentTrackingError::Set(_error) => None,
      DeploymentTrackingError::Delete(_error) => None,
    }
  }
}

impl fmt::Display for DeploymentTrackingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DeploymentTrackingError::Other(message) => write!(f, "{}", message),
      DeploymentTrackingError::DatabaseNotFound => write!(f, "Couldn't find tracking database"),
      DeploymentTrackingError::Deserialization(error) => write!(f, "Couldn't deserialize tracking data: {}", error),
      DeploymentTrackingError::Serialization(error) => write!(f, "Couldn't serialize tracking data: {}", error),
      DeploymentTrackingError::Insertion(error) => write!(f, "Couldn't insert tracking data before writing to disc: {}", error),
      DeploymentTrackingError::Read(error) => write!(f, "Couldn't read tracking data: {}", error),
      DeploymentTrackingError::Set(error) => write!(f, "Couldn't set tracking data: {}", error),
      DeploymentTrackingError::Delete(error) => write!(f, "Couldn't delete tracking data: {}", error),
    }
  }
}

impl From<io::Error> for DeploymentTrackingError {
  fn from(error: io::Error) -> Self {
    DeploymentTrackingError::Other(error.to_string())
  }
}

impl From<toml::de::Error> for DeploymentTrackingError {
  fn from(error: toml::de::Error) -> Self {
    DeploymentTrackingError::Deserialization(error)
  }
}

impl From<toml::ser::Error> for DeploymentTrackingError {
  fn from(error: toml::ser::Error) -> Self {
    DeploymentTrackingError::Serialization(error)
  }
}

impl From<toml_query::error::Error> for DeploymentTrackingError {
  fn from(error: toml_query::error::Error) -> Self {
    DeploymentTrackingError::Other(error.to_string())
  }
}

impl From<blockchain::error::ConnectionError> for DeploymentTrackingError {
  fn from(error: blockchain::error::ConnectionError) -> Self {
    DeploymentTrackingError::Other(error.to_string())
  }
}
