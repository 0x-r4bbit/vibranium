use std::process::{Command, Child};

pub struct NodeConfig<'a> {
  pub client: &'a str,
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

  pub fn start(&self) -> Result<Child, std::io::Error> {
    return Command::new(self.config.client).spawn();
  }
}


