pub mod blockchain;

use std::process::ExitStatus;

#[derive(Debug)]
pub struct Vibranium;

impl Vibranium {
  pub fn new() -> Vibranium {
    Vibranium {
    }
  }

  pub fn start_node(&self, config: blockchain::NodeConfig) -> Result<ExitStatus, std::io::Error> {
    let node = blockchain::Node::new(config);
    node.start()
        .map(|mut process| process.wait())
        .and_then(|status| status)
  }
}
