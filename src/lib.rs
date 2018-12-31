use std::process::{Command};

#[derive(Debug)]
pub struct Vibranium;

impl Vibranium {
  pub fn new() -> Vibranium {
    Vibranium {
    }
  }

  pub fn start_node(&self) {
    let mut cmd = Command::new("parity").spawn().unwrap();
    cmd.wait();
  }
}
