extern crate vibranium;
extern crate toml;

use std::error::Error;
use std::fmt;

use vibranium::compiler::error::CompilerError;
use vibranium::config::error::ConfigError;
use vibranium::blockchain::error::NodeError;
use vibranium::blockchain::error::ConnectionError;
use vibranium::deployment::error::DeploymentError;


const ERROR_MESSAGE_CONNECTION_REFUSED: &str = "Unable to connect to blockchain. If you're trying to connect to a local blockchain node,
make sure it's started first using the following command in a separate process:

  $ vibranium node [--path ...]
";

#[derive(Debug)]
pub enum CliError {
  CompilationError(CompilerError),
  ConfigurationSetError(toml::ser::Error),
  ConfigurationDeleteError(ConfigError),
  BlockchainError(NodeError),
  BlockchainConnectorError(ConnectionError),
  DeploymentError(DeploymentError),
  Other(String),
}

impl Error for CliError {
  fn cause(&self) -> Option<&dyn Error> {
    match self {
      CliError::CompilationError(error) => Some(error),
      CliError::ConfigurationSetError(error) => Some(error),
      CliError::ConfigurationDeleteError(error) => Some(error),
      CliError::BlockchainError(error) => Some(error),
      CliError::BlockchainConnectorError(error) => Some(error),
      CliError::DeploymentError(error) => Some(error),
      CliError::Other(_message) => None,
    }
  }
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CliError::CompilationError(error) => {
        match error {
          CompilerError::UnsupportedStrategy => {
            write!(f, "No built-in support for requested compiler.
To use this compiler, please specify necessary OPTIONS in compile command. E.g:

  vibranium compile --compiler solcjs -- <OPTIONS>...

OPTIONS can also be specified in the project's vibranium.toml file:

  [compiler]
    options = [\"--option1\", \"--option2\"]
")
          }
          _ => write!(f, "{}", error),
        }
      },
      CliError::ConfigurationSetError(error) => write!(f, "Couldn't set configuration: {}", error),
      CliError::ConfigurationDeleteError(_error) => write!(f, "Cannot delete config array or object option that isn't empty"),
      CliError::BlockchainError(error) => {
        match error {
          NodeError::UnsupportedClient => {
            write!(f, "No built-in support for requested blockchain client.
To use this client, please specify necessary OPTIONS in the node command. E.g:

  vibranium node --client trinity -- <OPTIONS>...

OPTIONS can also be specified in the project's vibranium.toml file:

  [blockchain]
    options = [\"--option1\", \"--option2\"]
")
          },
          _ => write!(f, "{}", error),
        }
      },
      CliError::BlockchainConnectorError(error) => {
        match error {
          ConnectionError::MissingConnectorConfig => {
            write!(f, "Unable to connect to blockchain. Couldn't find blockchain connector configuration in project configuration.
Make sure a blockchain connector configuration is provided in the project's vibranium.toml file. E.g:

  [blockchain.connector]
    protocol = \"ws\"
    port = \"8546\"
    host = \"127.0.0.1\"
")
          },
          ConnectionError::Transport(error) => {
            // Unfortunately, the underlying web3::Error doesn't properly
            // expose its error kinds, so we have to rely on string parsing
            // to transform them to meaningful error messages.
            let error_message = error.to_string();
            if error_message.contains("Connection refused") {
              write!(f, "{}", ERROR_MESSAGE_CONNECTION_REFUSED)
            } else if error_message.contains("invalid response") || error_message.contains("405 Method Not Allowed") {
              write!(f, "Unexpected blockchain client response. This is because of either a bad reponse from the blockchain client,
or a wrong configuration of the blockchain connector (e.g. wrong port)")
            } else {
              write!(f, "{}", error)
            }
          },
          _ => write!(f, "{}", error)
        }
      },
      CliError::DeploymentError(error) => {
        match error {
          DeploymentError::MissingConfig => {
            write!(f, "Unable to deploy Smart Contract. Couldn't find deployment configuration in project configuration.
Make sure a deployment configuration is provided in the project's vibranium.toml file. E.g.:

  [deployment]

    [[deployment.smart_contracts]]
      name = \"SmartContractName\"
      args = [ {{ value = \"somevalue\", kind = \"string\" }} ]
")
          },
          _ => write!(f, "{}", error)
        }
      },
      CliError::Other(message) => {

        if message.contains("Connection refused") {
          write!(f, "{}", ERROR_MESSAGE_CONNECTION_REFUSED)
        } else {
          write!(f, "{}", message)
        }
      },
    }
  }
}


