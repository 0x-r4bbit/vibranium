use super::error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

use crate::project_generator;
use project_generator::{
  DEFAULT_DATADIR_NAME,
  DEFAULT_ENVIRONMENT,
  DEFAULT_DEV_PASSWORDS_DIR,
};

const PARITY_CLIENT_BINARY_UNIX: &str = "parity";
const PARITY_CLIENT_BINARY_WINDOWS: &str = "parity.exe";
const GETH_CLIENT_BINARY_UNIX: &str = "geth";
const GETH_CLIENT_BINARY_WINDOWS: &str = "geth.exe";
const GANACHE_CLIENT_BINARY: &str = "ganache-cli";

const PARITY_DEFAULT_DEV_ACCOUNT: &str = "0x00a329c0648769a73afac7f9381e08fb43dbea72";
const PARITY_PASSWORDS_FILE: &str = "parity_dev";

pub enum SupportedBlockchainClients {
  Parity,
  Geth,
  Ganache,
}

impl SupportedBlockchainClients {
  pub fn executable(&self) -> String {
    match self {
      SupportedBlockchainClients::Parity => {
        if cfg!(target_os = "windows") {
          PARITY_CLIENT_BINARY_WINDOWS.to_string()
        } else {
          PARITY_CLIENT_BINARY_UNIX.to_string()
        }
      }
      SupportedBlockchainClients::Geth => {
        if cfg!(target_os = "windows") {
          GETH_CLIENT_BINARY_WINDOWS.to_string()
        } else {
          GETH_CLIENT_BINARY_UNIX.to_string()
        }
      },
      SupportedBlockchainClients::Ganache => GANACHE_CLIENT_BINARY.to_string()
    }
  }
}

impl FromStr for SupportedBlockchainClients {
  type Err = error::NodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      PARITY_CLIENT_BINARY_UNIX => Ok(SupportedBlockchainClients::Parity),
      GETH_CLIENT_BINARY_UNIX => Ok(SupportedBlockchainClients::Geth),
      GANACHE_CLIENT_BINARY => Ok(SupportedBlockchainClients::Ganache),
      _ => Err(error::NodeError::UnsupportedClient),
    }
  }
}

impl ToString for SupportedBlockchainClients {
  fn to_string(&self) -> String {
    match self {
      SupportedBlockchainClients::Parity => PARITY_CLIENT_BINARY_UNIX.to_string(),
      SupportedBlockchainClients::Geth => GETH_CLIENT_BINARY_UNIX.to_string(),
      SupportedBlockchainClients::Ganache => GANACHE_CLIENT_BINARY.to_string(),
    }
  }
}

pub fn init_node(client: &str, options: &Vec<String>, vibranium_dir_path: &PathBuf) -> Result<(), std::io::Error> {
  match client.parse() {
    Ok(SupportedBlockchainClients::Parity) => {

      let password_option = "--password".to_string();

      if options.contains(&password_option) {
        let i = options.iter().position(|v| v == &password_option).unwrap() + 1;
        let default_password_dir = vibranium_dir_path.join(DEFAULT_DEV_PASSWORDS_DIR);
        let default_password_file = default_password_dir.join(PARITY_PASSWORDS_FILE);

        if options[i].ends_with(&default_password_file.to_str().unwrap()) && !default_password_file.exists() {
          fs::create_dir_all(&default_password_dir)?;
          let mut f = fs::File::create(&default_password_file)?;
          f.write_all("\ndev_password".to_string().as_bytes())?
        }
      }

      Ok(())
    },
    Ok(SupportedBlockchainClients::Geth) => Ok(()),
    Ok(SupportedBlockchainClients::Ganache) => Ok(()),
    Err(_) => Ok(()),
  }
}

pub fn default_options_from(client: SupportedBlockchainClients, vibranium_dir_path: &PathBuf) -> Vec<String> {
  match client {
    SupportedBlockchainClients::Parity => {
      vec![
        "--config".to_string(),
        "dev".to_string(),
        "--ws-origins".to_string(),
        "all".to_string(),
        "--base-path".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DATADIR_NAME)
          .join(DEFAULT_ENVIRONMENT)
          .to_string_lossy()
          .to_string(),
        "--unlock".to_string(),
        PARITY_DEFAULT_DEV_ACCOUNT.to_string(),
        "--password".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DEV_PASSWORDS_DIR)
          .join(PARITY_PASSWORDS_FILE)
          .to_string_lossy()
          .to_string(),
      ]
    },
    SupportedBlockchainClients::Geth => {
      vec![
        "--dev".to_string(),
        "--rpc".to_string(),
        "--ws".to_string(),
        "--wsorigins".to_string(),
        "*".to_string(),
        "--datadir".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DATADIR_NAME)
          .join(DEFAULT_ENVIRONMENT)
          .to_string_lossy()
          .to_string(),
      ]
    },
    SupportedBlockchainClients::Ganache => {
      vec![
        "--deterministic".to_string(),
        "--db".to_string(),
        vibranium_dir_path
          .join(DEFAULT_DATADIR_NAME)
          .join(DEFAULT_ENVIRONMENT)
          .to_string_lossy()
          .to_string(),
      ]
    }
  }
}
