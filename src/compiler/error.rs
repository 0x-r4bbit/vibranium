use std::error::Error;
use std::fmt;
use std::io;

use crate::config;
use crate::project_generator;

#[derive(Debug)]
pub enum CompilerError {
  Io(io::Error),
  VibraniumDirectoryNotFound(project_generator::error::ProjectGenerationError),
  InvalidConfig(config::error::ConfigError),
  UnsupportedStrategy,
  Other(String),
}

impl Error for CompilerError {
  fn description(&self) -> &str {
    let executable_not_found_message = "Couldn't find executable for requested compiler";
    match self {
      CompilerError::Io(error) => {
        match error.kind() {
          io::ErrorKind::NotFound => executable_not_found_message,
          _ => error.description(),
        }
      },
      CompilerError::VibraniumDirectoryNotFound(error) => error.description(),
      CompilerError::InvalidConfig(error) => error.description(),
      CompilerError::UnsupportedStrategy => "Couldn't compile project without `CompilerConfig::compiler_options`. No built-in support for requested compiler.",
      CompilerError::Other(message) => {
        if message.contains("not found") || message.contains("not recognized") {
          executable_not_found_message
        } else {
          message
        }
      },
    }
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      CompilerError::Io(error) => Some(error),
      CompilerError::VibraniumDirectoryNotFound(error) => Some(error),
      CompilerError::InvalidConfig(error) => Some(error),
      CompilerError::UnsupportedStrategy => None,
      CompilerError::Other(_message) => None,
    }
  }
}

impl fmt::Display for CompilerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CompilerError::Io(_error) => write!(f, "{}", self.description()),
      CompilerError::VibraniumDirectoryNotFound(error) => write!(f, "{}", error),
      CompilerError::InvalidConfig(error) => write!(f, "{}", error),
      CompilerError::UnsupportedStrategy => write!(f, "{}", self.description()),
      CompilerError::Other(_message) => write!(f, "{}", self.description()),
    }
  }
}
