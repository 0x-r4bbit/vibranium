use super::Strategy;
use std::process::{Command, Child, Stdio};

pub struct DefaultStrategy {
  pub compiler_bin: String,
  pub compiler_options: Vec<String>,
}

impl DefaultStrategy {
  pub fn new(compiler_bin: String, compiler_options: Vec<String>) -> DefaultStrategy {
    DefaultStrategy {
      compiler_bin,
      compiler_options
    }
  }
}

impl Strategy for DefaultStrategy {
  fn execute(&self) -> Result<Child, std::io::Error> {
    info!("Compiling project using command: {} {}", &self.compiler_bin, self.compiler_options.join(" "));
    Command::new(&self.compiler_bin)
      .args(&self.compiler_options)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
  }
}
