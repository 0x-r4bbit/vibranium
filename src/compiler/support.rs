use super::error;

use std::str::FromStr;
use std::string::ToString;

const SOLC_COMPILER_BINARY: &str = "solc";
const SOLC_JS_COMPILER_BINARY: &str = "solcjs";

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

pub fn default_options_from(compiler: SupportedCompilers) -> Vec<String> {
  match compiler {
    SupportedCompilers::Solc => {
      vec![
        "--abi".to_string(),
        "--bin".to_string(),
        "--overwrite".to_string(),
        "-o".to_string()
      ]
    },
    SupportedCompilers::SolcJs => {
      vec![
        "--abi".to_string(),
        "--bin".to_string(),
        "-o".to_string()
      ]
    },
  }
}
