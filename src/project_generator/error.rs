extern crate toml;

use std::error::Error;
use std::fmt;
use std::io;
use crate::config;

#[derive(Debug)]
pub enum ProjectGenerationError {
  InvalidConfig(config::error::ConfigError),
  ProjectPathNotFound,
  VibraniumDirectoryNotFound,
  Io(io::Error),
  Serialization(toml::ser::Error),
}

impl Error for ProjectGenerationError {
  fn cause(&self) -> Option<&Error> {
    match self {
      ProjectGenerationError::InvalidConfig(error) => Some(error),
      ProjectGenerationError::ProjectPathNotFound => None,
      ProjectGenerationError::VibraniumDirectoryNotFound => None,
      ProjectGenerationError::Io(error) => Some(error),
      ProjectGenerationError::Serialization(error) => Some(error),
    }
  }
}

impl fmt::Display for ProjectGenerationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ProjectGenerationError::InvalidConfig(error) => write!(f, "{}", error),
      ProjectGenerationError::ProjectPathNotFound => write!(f, "Couldn't find directory for given project path"),
      ProjectGenerationError::VibraniumDirectoryNotFound => write!(f, "Not a Vibranium project. Couldn't find .vibranium directory"),
      ProjectGenerationError::Io(error) => write!(f, "{}", error),
      ProjectGenerationError::Serialization(error) => write!(f, "Couldn't serialize data: {}", error),
    }
  }
}
