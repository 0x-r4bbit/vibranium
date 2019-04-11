use super::error;
use super::strategy;

use std::str::FromStr;
use std::string::ToString;
use strategy::solc::SOLC_COMPILER_BINARY;
use strategy::solcjs::SOLC_JS_COMPILER_BINARY;

pub enum SupportedCompilers {
  Solc,
  SolcJs,
}

impl FromStr for SupportedCompilers {
  type Err = error::CompilerError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      SOLC_COMPILER_BINARY => Ok(SupportedCompilers::Solc),
      SOLC_JS_COMPILER_BINARY => Ok(SupportedCompilers::SolcJs),
      _ => Err(error::CompilerError::UnsupportedStrategy),
    }
  }
}

impl ToString for SupportedCompilers {
  fn to_string(&self) -> String {
    match self {
      SupportedCompilers::Solc => SOLC_COMPILER_BINARY.to_string(),
      SupportedCompilers::SolcJs => SOLC_JS_COMPILER_BINARY.to_string(),
    }
  }
}


