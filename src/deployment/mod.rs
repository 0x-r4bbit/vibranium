pub mod error;

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
use web3::types::{U256, Address};
use error::DeploymentError;

const ARTIFACT_EXTENSION_BINARY: &str = "bin";
const ARTIFACT_EXTENSION_ABI: &str = "abi";
const DEFAULT_GAS_PRICE: usize = 5;
const DEFAULT_GAS_LIMIT: usize = 2_000_000;
const DEFAULT_DEV_TX_CONFIRMATION_AMOUNT: usize = 0;

pub struct Deployer<'a> {
  config: &'a Config,
  connector: &'a BlockchainConnector
}

impl<'a> Deployer<'a> {
  pub fn new(config: &'a Config, connector: &'a BlockchainConnector) -> Deployer<'a> {
    Deployer {
      config,
      connector
    }
  }

  pub fn deploy(&self) -> Result<HashMap<String, (String, Address)>, DeploymentError>  {

    let project_config = self.config.read().map_err(|err| DeploymentError::Other(err.to_string()))?;

    if let None = &project_config.deployment {
      return Err(DeploymentError::MissingConfig);
    }

    let deployment_config = &project_config.deployment.unwrap();

    let artifacts_path = self.config.project_path.join(&project_config.sources.artifacts);
    let mut artifacts_dir = std::fs::read_dir(&artifacts_path).map_err(|err| DeploymentError::Other(err.to_string()))?;
    let accounts = self.connector.accounts().map_err(DeploymentError::Connection)?;

    let general_gas_price = deployment_config.gas_price.map(U256::from).unwrap_or_else(|| self.connector.gas_price().ok().unwrap_or_else(|| U256::from(DEFAULT_GAS_PRICE)));
    let general_gas_limit = deployment_config.gas_limit.map(U256::from).unwrap_or(U256::from(DEFAULT_GAS_LIMIT));

    let confirmations = deployment_config.tx_confirmations.unwrap_or(DEFAULT_DEV_TX_CONFIRMATION_AMOUNT);
    let mut deployed_contracts = HashMap::new();

    for smart_contract_config in &deployment_config.smart_contracts {

      if let Some(artifact) = artifacts_dir.find(|entry| {
        entry.as_ref().unwrap().path().to_string_lossy().to_string().contains(&smart_contract_config.name)
      }) {
        let artifact = artifact.map_err(|err| DeploymentError::Other(err.to_string()))?;
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

          info!("Deploying {}...", &smart_contract_config.name);

          let mut builder = self.connector.deploy(&abi).map_err(|err| {
            println!("{:?}", err);
            DeploymentError::Other(err.to_string())
          })?;

          builder = builder.confirmations(confirmations)
                                .options(Options::with(|opts| {
                                  opts.gas_price = smart_contract_config.gas_price.map(U256::from).or(Some(general_gas_price));
                                  opts.gas = smart_contract_config.gas_limit.map(U256::from).or(Some(general_gas_limit));
                                }));

          let pending_contract = match args.iter().count() {
            0 => builder.execute(bytecode, (), accounts[0]),
            1 => builder.execute(bytecode, args[0].to_owned(), accounts[0]),
            2 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned()), accounts[0]),
            3 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned()), accounts[0]),
            4 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned()), accounts[0]),
            5 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned()), accounts[0]),
            6 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned(), args[5].to_owned()), accounts[0]),
            7 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned(), args[5].to_owned(), args[6].to_owned()), accounts[0]),
            8 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned(), args[5].to_owned(), args[6].to_owned(), args[7].to_owned()), accounts[0]),
            9 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned(), args[5].to_owned(), args[6].to_owned(), args[7].to_owned(),
                                            args[8].to_owned()), accounts[0]),
            10 => builder.execute(bytecode, (args[0].to_owned(), args[1].to_owned(), args[2].to_owned(), args[3].to_owned(),
                                            args[4].to_owned(), args[5].to_owned(), args[6].to_owned(), args[7].to_owned(),
                                            args[8].to_owned(), args[9].to_owned()), accounts[0]),
            _ => return Err(DeploymentError::TooManyConstructorArgs(smart_contract_config.name.to_owned())),
          };
          
          let pending_contract = pending_contract.map_err(|err| DeploymentError::InvalidConstructorArgs(err, smart_contract_config.name.to_owned()))?;
          let contract = pending_contract.wait().map_err(|err| DeploymentError::DeployContract(err, smart_contract_config.name.to_owned()))?;

          info!("Deployed {} at {:?}", &smart_contract_config.name, &contract.address());

          deployed_contracts.insert(file_bin_path.to_string_lossy().to_string(), (smart_contract_config.name.to_owned(), contract.address()));
        }
      }
    }
    Ok(deployed_contracts)
  }
}

fn tokenize_args(args: &Vec<SmartContractArg>) -> Result<Vec<Token>, DeploymentError> {
  let mut tokenized_args: Vec<Token> = vec![];

  for arg in args {
    let param_type = Reader::read(&arg.kind).map_err(DeploymentError::InvalidParamType)?;
    let token = LenientTokenizer::tokenize(&param_type, &arg.value).map_err(|err| DeploymentError::TokenizeParam(err, arg.value.to_owned()))?;
    tokenized_args.push(token);
  }

  Ok(tokenized_args)
}
