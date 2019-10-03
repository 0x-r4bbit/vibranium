pub mod error;

use crate::blockchain::connector::BlockchainConnector;
use web3::types::Address;

pub struct AccountsManager<'a> {
    connector: &'a BlockchainConnector,
}

impl<'a> AccountsManager<'a> {
    pub fn new(connector: &'a BlockchainConnector) -> AccountsManager<'a> {
        AccountsManager { connector }
    }

    pub fn get_node_accounts(&self) -> Result<Vec<Address>, error::AccountsError> {
        self.connector
            .accounts()
            .map_err(|err| error::AccountsError::Other(err.to_string()))
    }
}
