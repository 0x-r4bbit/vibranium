use std::error::Error;
use std::fmt;

use crate::blockchain;

#[derive(Debug)]
pub enum DeploymentError {
  MissingConfig,
  InvalidParamType(ethabi::Error),
  TokenizeParam(ethabi::Error, String),
  NothingToDeploy,
  MissingArtifact(String, String),
  TooManyConstructorArgs(String),
  Connection(blockchain::error::ConnectionError),
  DeployContract(web3::contract::deploy::Error, String),
  InvalidConstructorArgs(ethabi::Error, String),
  Other(String),
}

impl Error for DeploymentError {
  fn cause(&self) -> Option<&Error> {
    match self {
      DeploymentError::MissingConfig => None,
      DeploymentError::InvalidParamType(error) => Some(error),
      DeploymentError::TokenizeParam(error, _value) => Some(error),
      DeploymentError::NothingToDeploy => None,
      DeploymentError::MissingArtifact(_kind, _name) => None,
      DeploymentError::TooManyConstructorArgs(_name) => None,
      DeploymentError::Connection(error) => Some(error),
      DeploymentError::DeployContract(error, _name) => Some(error),
      DeploymentError::InvalidConstructorArgs(error, _name) => Some(error),
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
      DeploymentError::MissingArtifact(kind, name) => write!(f, "Couldn't find {} file for artifact '{}'", kind, name),
      DeploymentError::TooManyConstructorArgs(name) => write!(f, "Couldn't deploy Smart Contract '{}' due to too many constructor arguments (max. 10)", name),
      DeploymentError::Connection(error) => write!(f, "{}", error),
      DeploymentError::DeployContract(error, name) => write!(f, "Couldn't deploy Smart Contract '{}' due to {}", name, error),
      DeploymentError::InvalidConstructorArgs(_error, name) => write!(f, "Couldn't deploy Smart Contract '{}' due to mismatching types in constructor arguments.", name),
      DeploymentError::Other(message) => write!(f, "{}", message),
    }
  }
}
