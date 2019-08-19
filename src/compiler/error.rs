use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::config;
use crate::project_generator;

#[derive(Debug)]
pub enum CompilerError {
  Io(io::Error),
  ExecutableNotFound(io::Error, String),
  VibraniumDirectoryNotFound(project_generator::error::ProjectGenerationError),
  InvalidConfig(config::error::ConfigError),
  ImportError(PathBuf),
  UnsupportedStrategy,
  Other(String),
}

impl Error for CompilerError {
  fn cause(&self) -> Option<&Error> {
    match self {
      CompilerError::Io(error) => Some(error),
      CompilerError::ExecutableNotFound(error, _exec) => Some(error),
      CompilerError::VibraniumDirectoryNotFound(error) => Some(error),
      CompilerError::InvalidConfig(error) => Some(error),
      CompilerError::UnsupportedStrategy => None,
      CompilerError::ImportError(_path) => None,
      CompilerError::Other(_message) => None,
    }
  }
}

impl fmt::Display for CompilerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CompilerError::Io(error) => write!(f, "{}", error.description()),
      CompilerError::ExecutableNotFound(_error, exec) => write!(f, "Couldn't find executable for compiler {}", exec),
      CompilerError::VibraniumDirectoryNotFound(error) => write!(f, "{}", error.description()),
      CompilerError::InvalidConfig(error) => write!(f, "{}", error.description()),
      CompilerError::UnsupportedStrategy => write!(f, "Couldn't compile project without `CompilerConfig::compiler_options`. No built-in support for requested compiler."),
      CompilerError::ImportError(path) => write!(f, "Couldn't compile project. Import file doesn't exist: {:?}", path),
      CompilerError::Other(message) => write!(f, "{}", &message),
    }
  }
}

impl From<config::error::ConfigError> for CompilerError {
  fn from(error: config::error::ConfigError) -> Self {
    match error {
      config::error::ConfigError::Deserialization(_) => CompilerError::InvalidConfig(error),
      _ => CompilerError::Other(error.to_string()),
    }
  }
}

impl From<std::io::Error> for CompilerError {
  fn from(error: std::io::Error) -> Self {
    CompilerError::Io(error)
  }
}
