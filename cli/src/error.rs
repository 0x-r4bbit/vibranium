extern crate vibranium;

use std::error::Error;
use std::fmt;

use vibranium::compiler::error::CompilerError;

#[derive(Debug)]
pub enum CliError {
  CompilationError(CompilerError),
}

impl Error for CliError {
  fn description(&self) -> &str {
    match self {
      CliError::CompilationError(error) => {
        match error {
          CompilerError::UnsupportedStrategy => r###"No built-in support for requested compiler.
To use this compiler, specify ARGS in compile command. E.g:

  vibranium compile --compiler solcjs -- <ARGS>..."###,
          _ => error.description()

        }
      } 
    }
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      CliError::CompilationError(error) => Some(error),
    }
  }
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CliError::CompilationError(_error) => write!(f, "{}", self.description()),
    }
  }
}


