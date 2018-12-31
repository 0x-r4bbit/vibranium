pub mod blockchain;

#[derive(Debug)]
pub struct Vibranium;

impl Vibranium {
  pub fn new() -> Vibranium {
    Vibranium {
    }
  }

  pub fn start_node(&self, config: blockchain::NodeConfig) {
    let node = blockchain::Node::new(config);
    let process = node.start();
    process.unwrap().wait();
  }
}
