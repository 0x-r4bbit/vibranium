extern crate toml;

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ProjectGenerationError {
  ConfigSerialization(toml::ser::Error),
  ConfigDeserialization(toml::de::Error),
  ProjectPathNotFound,
  VibraniumDirectoryNotFound,
  Io(io::Error),
}

impl Error for ProjectGenerationError {
  fn description(&self) -> &str {
    let description = match self {
      ProjectGenerationError::ConfigSerialization(error) => error.description(),
      ProjectGenerationError::ConfigDeserialization(error) => error.description(),
      ProjectGenerationError::ProjectPathNotFound => "Couldn't find directory for given project path",
      ProjectGenerationError::VibraniumDirectoryNotFound => "Couldn't find .vibranium directory",
      ProjectGenerationError::Io(error) => error.description(),
    };
    return description
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      ProjectGenerationError::ConfigSerialization(error) => Some(error),
      ProjectGenerationError::ConfigDeserialization(error) => Some(error),
      ProjectGenerationError::ProjectPathNotFound => None,
      ProjectGenerationError::VibraniumDirectoryNotFound => None,
      ProjectGenerationError::Io(error) => Some(error),
    }
  }
}

impl fmt::Display for ProjectGenerationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ProjectGenerationError::ConfigSerialization(error) => write!(f, "Couldn't serialize vibranium config: {}", error),
      ProjectGenerationError::ConfigDeserialization(error) => write!(f, "Couldn't deserialize vibranium config: {}", error),
      ProjectGenerationError::ProjectPathNotFound => write!(f, "{}", self.description()),
      ProjectGenerationError::VibraniumDirectoryNotFound => write!(f, "Not a Vibranium project: {}", self.description()),
      ProjectGenerationError::Io(error) => write!(f, "{}", error),
    }
  }
}
