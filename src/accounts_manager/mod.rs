pub mod error;

use crate::blockchain::connector::BlockchainConnector;
use crate::config::{Config, AccountConfig};
use std::str::FromStr;
use wagyu_ethereum::EthereumPrivateKey;
use wagyu_model::private_key::PrivateKey;
use web3::types::Address;

pub struct AccountsManager<'a> {
  config: &'a Config,
  connector: &'a BlockchainConnector,
}

impl<'a> AccountsManager<'a> {
  pub fn new(config: &'a Config, connector: &'a BlockchainConnector) -> AccountsManager<'a> {
    AccountsManager {
      config,
      connector,
    }
  }

  pub fn get_node_accounts(&self) -> Result<Vec<Address>, error::AccountsError> {
    self.connector.accounts().map_err(|err| error::AccountsError::Other(err.to_string()))
  }

  pub fn get_wallet_accounts(&self, accounts: Vec<AccountConfig>) -> Result<Vec<Address>, error::AccountsError> {
    let mut wallet_accounts: Vec<Address> = vec![];

    for account_config in accounts {
      if account_config.private_key.is_some() {
        let private_key = EthereumPrivateKey::from_str(&account_config.private_key.unwrap()).unwrap();
        wallet_accounts.push(private_key.to_address().unwrap());
      }
    }

    Ok(wallet_accounts)
  }
}
