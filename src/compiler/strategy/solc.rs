use super::{Strategy, StrategyConfig};
use std::process::{Command, Child, Stdio};
use glob::glob;

pub const SOLC_COMPILER_BINARY: &str = "solc";

pub struct SolcStrategy<'a> {
  config: StrategyConfig<'a>
}

impl<'a> SolcStrategy<'a> {
  pub fn new(config: StrategyConfig) -> SolcStrategy {
    SolcStrategy {
      config
    }
  }
}

impl<'a> Strategy for SolcStrategy<'a> {
  fn execute(&self) -> Result<Child, std::io::Error> {

    let mut args: Vec<String> = vec![
      "--abi".to_string(),
      "--metadata".to_string(),
      "--userdoc".to_string(),
      "--overwrite".to_string(),
      "-o".to_string(),
      self.config.output_path.to_string_lossy().to_string()
    ];

    for pattern in &self.config.smart_contract_sources {
      let mut full_pattern = self.config.input_path.clone();
      full_pattern.push(&pattern);
      for entry in glob(&full_pattern.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        args.push(entry.to_string_lossy().to_string());
      }
    }

    Command::new(SOLC_COMPILER_BINARY)
      .args(args)
      .args(&self.config.compiler_options)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
  }
}

