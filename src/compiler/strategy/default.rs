use super::Strategy;
use std::process::{Command, Child, Stdio};

pub struct DefaultStrategy<'a> {
  pub compiler_bin: &'a str,
  pub compiler_options: Vec<&'a str>,
}

impl<'a> DefaultStrategy<'a> {
  pub fn new(compiler_bin: &'a str, compiler_options: Vec<&'a str>) -> DefaultStrategy<'a> {
    DefaultStrategy {
      compiler_bin,
      compiler_options
    }
  }
}

impl<'a> Strategy for DefaultStrategy<'a> {
  fn execute(&self) -> Result<Child, std::io::Error> {
    Command::new(&self.compiler_bin)
      .args(&self.compiler_options)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
  }
}
