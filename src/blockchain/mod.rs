extern crate log;

use std::process::{Command, Child};

pub mod error;

pub struct NodeConfig<'a> {
  pub client: &'a str,
  pub client_options: &'a Vec<&'a str>,
}

pub struct Node<'a> {
  config: NodeConfig<'a>,
}

impl<'a> Node<'a> {
  pub fn new(config: NodeConfig) -> Node {
    Node {
      config,
    }
  }

  pub fn start(&self) -> Result<Child, error::NodeError> {
    info!("Starting node with command: {} {}", &self.config.client, self.config.client_options.join(" "));
    Command::new(self.config.client)
            .args(self.config.client_options)
            .spawn()
            .map_err(error::NodeError::Io)
  }
}



