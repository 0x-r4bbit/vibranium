pub mod error;
pub mod tracker;

use std::fs;
use std::path::Path;
use std::collections::HashMap;
use ethabi::{Token};
use ethabi::param_type::Reader;
use ethabi::token::{LenientTokenizer, Tokenizer};
use crate::blockchain;
use crate::config;
use blockchain::connector::{BlockchainConnector};
use config::{Config, SmartContractArg};
use web3::contract::Options;
use web3::futures::Future;
use web3::types::{U256, H256, Address, BlockId, BlockNumber};
use error::DeploymentError;
use tracker::DeploymentTracker;

const ARTIFACT_EXTENSION_BINARY: &str = "bin";
const ARTIFACT_EXTENSION_ABI: &str = "abi";
const DEFAULT_GAS_PRICE: usize = 5;
const DEFAULT_GAS_LIMIT: usize = 2_000_000;
const DEFAULT_DEV_TX_CONFIRMATION_AMOUNT: usize = 0;

pub struct DeployOptions {
  pub tracking_enabled: Option<bool>,
}

pub struct Deployer<'a> {
  config: &'a Config,
  connector: &'a BlockchainConnector,
  tracker: &'a DeploymentTracker<'a>,
}

impl<'a> Deployer<'a> {
  pub fn new(config: &'a Config, connector: &'a BlockchainConnector, tracker: &'a DeploymentTracker) -> Deployer<'a> {
    Deployer {
      config,
      connector,
      tracker,
    }
  }

  pub fn deploy(&self, options: DeployOptions) -> Result<HashMap<String, (String, Address, bool)>, DeploymentError>  {

    let project_config = self.config.read()?;

    if project_config.deployment.is_none() {
      return Err(DeploymentError::MissingConfig);
    }

    let deployment_config = &project_config.deployment.unwrap();

    let artifacts_path = self.config.project_path.join(&project_config.sources.artifacts);
    let mut artifacts_dir = std::fs::read_dir(&artifacts_path)?;
    let accounts = self.connector.accounts()?;

    let general_gas_price = deployment_config.gas_price.map(U256::from).unwrap_or_else(|| self.connector.gas_price().ok().unwrap_or_else(|| U256::from(DEFAULT_GAS_PRICE)));
    let general_gas_limit = deployment_config.gas_limit.map(U256::from).unwrap_or_else(|| U256::from(DEFAULT_GAS_LIMIT));

    let confirmations = deployment_config.tx_confirmations.unwrap_or(DEFAULT_DEV_TX_CONFIRMATION_AMOUNT);
    let mut deployed_contracts = HashMap::new();

    let tracking_enabled = options.tracking_enabled
      .unwrap_or(deployment_config.tracking_enabled.unwrap_or(true));

    if tracking_enabled && !self.tracker.database_exists() {
      self.tracker.create_database()?;
    }

    for smart_contract_config in &deployment_config.smart_contracts {

      if let Some(artifact) = artifacts_dir.find(|entry| {
        entry.as_ref().unwrap().path().to_string_lossy().to_string().contains(&smart_contract_config.name)
      }) {
        let artifact = artifact?;
        let file_path = artifact.path();
        let file_extension = &file_path.extension().unwrap().to_str().unwrap();

        if file_extension == &ARTIFACT_EXTENSION_BINARY || file_extension == &ARTIFACT_EXTENSION_ABI {

          let file_bin_path = Path::new(&file_path).with_extension(&ARTIFACT_EXTENSION_BINARY);
          let file_abi_path = Path::new(&file_path).with_extension(&ARTIFACT_EXTENSION_ABI);

          if file_extension == &ARTIFACT_EXTENSION_BINARY && !file_abi_path.exists() {
            return Err(DeploymentError::MissingArtifact(ARTIFACT_EXTENSION_ABI.to_string(), file_bin_path.to_string_lossy().to_string()));
          } else if file_extension == &ARTIFACT_EXTENSION_ABI && !file_bin_path.exists() {
            return Err(DeploymentError::MissingArtifact(ARTIFACT_EXTENSION_BINARY.to_string(), file_abi_path.to_string_lossy().to_string()));
          }

          let bytecode = fs::read_to_string(&file_bin_path).unwrap();
          let abi = fs::read(file_abi_path).unwrap();

          let args = match &smart_contract_config.args {
            Some(args) => tokenize_args(args)?,
            None => vec![]
          };

          if tracking_enabled {
            let block_hash = self.get_first_block_hash().unwrap();
            let tracked_contract = self.tracker.get_tracking_data(&block_hash, &smart_contract_config.name, &bytecode, &args)?;

            if let Some(tracked_contract) = tracked_contract {
              info!("{} is already deployed at {}", &tracked_contract.name, &tracked_contract.address);
              deployed_contracts.insert(file_bin_path.to_string_lossy().to_string(), (tracked_contract.name, tracked_contract.address, true));
              continue;
            }
          }

          info!("Deploying {}...", &smart_contract_config.name);

          let builder = self.connector.deploy(&abi)?;

          let pending_contract = builder.confirmations(confirmations)
                                .options(Options::with(|opts| {
                                  opts.gas_price = smart_contract_config.gas_price.map(U256::from).or_else(|| Some(general_gas_price));
                                  opts.gas = smart_contract_config.gas_limit.map(U256::from).or_else(|| Some(general_gas_limit));
                                }))
                                .execute(&bytecode, &*args, accounts[0])
                                .map_err(|err| DeploymentError::InvalidConstructorArgs(err, smart_contract_config.name.to_owned()))?;

          let contract = pending_contract.wait().map_err(|err| DeploymentError::DeployContract(err, smart_contract_config.name.to_owned()))?;

          if tracking_enabled {
            self.tracker.track(
              self.get_first_block_hash().unwrap(), 
              smart_contract_config.name.to_owned(),
              bytecode,
              args,
              contract.address(),
            )?;
          }

          info!("Deployed {} at {:?}", &smart_contract_config.name, &contract.address());
          deployed_contracts.insert(file_bin_path.to_string_lossy().to_string(), (smart_contract_config.name.to_owned(), contract.address(), false));
        }
      }
    }
    Ok(deployed_contracts)
  }

  fn get_first_block_hash(&self) -> Result<H256, DeploymentError> {
    let block = self.connector.get_block(BlockId::Number(BlockNumber::Number(0)))?.unwrap();
    Ok(block.hash.unwrap())
  }
}

fn tokenize_args(args: &[SmartContractArg]) -> Result<Vec<Token>, DeploymentError> {
  let mut tokenized_args: Vec<Token> = vec![];

  for arg in args {
    let param_type = Reader::read(&arg.kind).map_err(DeploymentError::InvalidParamType)?;
    let token = LenientTokenizer::tokenize(&param_type, &arg.value).map_err(|err| DeploymentError::TokenizeParam(err, arg.value.to_owned()))?;
    tokenized_args.push(token);
  }

  Ok(tokenized_args)
}
