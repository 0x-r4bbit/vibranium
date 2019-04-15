use super::{Compile, StrategyConfig};
use std::process::{Command, Child, Stdio};
use glob::glob;

pub const SOLC_JS_COMPILER_BINARY: &str = "solcjs";

pub struct SolcJsStrategy {
  config: StrategyConfig
}

impl SolcJsStrategy {
  pub fn new(config: StrategyConfig) -> SolcJsStrategy {
    SolcJsStrategy {
      config
    }
  }
}

impl Compile for SolcJsStrategy {

  fn compile(&self) -> Result<Child, std::io::Error> {
    let mut compiler_options = vec!["--abi".to_string()];

    if let Some(options) = &self.config.compiler_options {
      compiler_options.append(&mut options.clone());
      compiler_options.sort();
      compiler_options.dedup();
    }

    compiler_options.push("-o".to_string());
    compiler_options.push(self.config.output_path.to_string_lossy().to_string());

    for pattern in &self.config.smart_contract_sources {
      let mut full_pattern = self.config.input_path.clone();
      full_pattern.push(&pattern);
      for entry in glob(&full_pattern.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        compiler_options.push(entry.to_string_lossy().to_string());
      }
    }

    info!("Compiling project using command: {} {}", SOLC_JS_COMPILER_BINARY, compiler_options.join(" "));

    Command::new(SOLC_JS_COMPILER_BINARY)
      .args(compiler_options)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
  }
}
