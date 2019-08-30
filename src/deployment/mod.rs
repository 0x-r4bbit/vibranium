pub mod error;
pub mod tracker;

use blockchain::connector::BlockchainConnector;
use config::{Config, SmartContractConfig, SmartContractArg};
use crate::blockchain;
use crate::config;
use error::DeploymentError;
use ethabi::{Token, ParamType};
use ethabi::param_type::Reader;
use ethabi::token::{LenientTokenizer, Tokenizer};
use petgraph::graphmap::DiGraphMap;
use petgraph::algo::toposort;
use std::fs;
use std::str::FromStr;
use std::path::PathBuf;
use std::collections::HashMap;
use tracker::DeploymentTracker;
use web3::contract::Options;
use web3::futures::Future;
use web3::types::{U256, H256, Address};

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

pub type DeployedContracts = HashMap<Address, (String, Address, String, bool)>;

impl<'a> Deployer<'a> {
  pub fn new(config: &'a Config, connector: &'a BlockchainConnector, tracker: &'a DeploymentTracker) -> Deployer<'a> {
    Deployer {
      config,
      connector,
      tracker,
    }
  }

  pub fn deploy(&self, options: DeployOptions) -> Result<DeployedContracts, DeploymentError>  {

    let project_config = self.config.read()?;

    if project_config.deployment.is_none() {
      return Err(DeploymentError::MissingConfig);
    }

    let deployment_config = &project_config.deployment.unwrap();
    let accounts = self.connector.accounts()?;

    let general_gas_price = deployment_config.gas_price.map(U256::from).unwrap_or_else(|| self.connector.gas_price().ok().unwrap_or_else(|| U256::from(DEFAULT_GAS_PRICE)));
    let general_gas_limit = deployment_config.gas_limit.map(U256::from).unwrap_or_else(|| U256::from(DEFAULT_GAS_LIMIT));

    let confirmations = deployment_config.tx_confirmations.unwrap_or(DEFAULT_DEV_TX_CONFIRMATION_AMOUNT);
    let mut deployed_contracts = HashMap::new();

    let tracking_enabled = options.tracking_enabled
      .unwrap_or_else(|| deployment_config.tracking_enabled.unwrap_or(true));

    if tracking_enabled && !self.tracker.database_exists() {
      self.tracker.create_database()?;
    }

    let sorted_smart_contract_configs = sort_by_dependencies(&deployment_config.smart_contracts)?;

    for smart_contract_config in sorted_smart_contract_configs {

      if let Some(address) = &smart_contract_config.address {
        let address = Address::from_str(&address[2..]).map_err(|err| DeploymentError::InvalidAddress(smart_contract_config.name.to_owned(), err.to_string()))?;
        info!("{} is already deployed at {:?}", &smart_contract_config.name, &address);
        deployed_contracts.insert(address, (smart_contract_config.name.clone(), address, "unknown".to_string(), true));
        continue;
      }

      if let Some((bin_path, abi_path)) = self.get_artifacts(&project_config.sources.artifacts, smart_contract_config)? {

        let bytecode = fs::read_to_string(&bin_path).unwrap();
        let abi = fs::read(abi_path).unwrap();

        let args = smart_contract_config.args.as_ref().unwrap_or(&vec![]).iter().map(|arg| arg.value.clone()).collect::<Vec<String>>();

        let tokenized_args = match &smart_contract_config.args {
          Some(args) => tokenize_args(args, &deployed_contracts)?,
          None => vec![]
        };

        if tracking_enabled {
          let block_hash = self.get_first_block_hash().unwrap();
          let tracked_contract = self.tracker.get_smart_contract_tracking_data(&block_hash, &smart_contract_config.name, &bytecode, &args)?;

          if let Some(tracked_contract) = tracked_contract {
            info!("{} is already deployed at {:?}", &tracked_contract.name, &tracked_contract.address);
            deployed_contracts.insert(tracked_contract.address, (tracked_contract.name, tracked_contract.address, bin_path.to_string_lossy().to_string(), true));
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
                              .execute(&bytecode, &*tokenized_args, accounts[0])
                              .map_err(|err| DeploymentError::InvalidConstructorArgs(err, smart_contract_config.name.to_owned()))?;

        let contract = pending_contract.wait().map_err(|err| DeploymentError::DeployContract(err, smart_contract_config.name.to_owned()))?;

        if tracking_enabled {
          self.tracker.track(
            self.get_first_block_hash().unwrap(), 
            smart_contract_config.name.to_owned(),
            bytecode,
            &args,
            contract.address(),
          )?;
        }

        info!("Deployed {} at {:?}", &smart_contract_config.name, &contract.address());
        deployed_contracts.insert(contract.address(), (smart_contract_config.name.to_owned(), contract.address(), bin_path.to_string_lossy().to_string(), false));
      } else {
        warn!("No bytecode or ABI found for Smart Contract '{}'", &smart_contract_config.name);
      }
    }
    Ok(deployed_contracts)
  }

  fn get_artifacts(&self, artifacts_path: &str, config: &SmartContractConfig) -> Result<Option<(PathBuf, PathBuf)>, DeploymentError> {
    if config.bytecode_path.is_some() && config.abi_path.is_none() {
      Err(DeploymentError::MissingABIPath(config.name.to_string()))
    } else if config.bytecode_path.is_none() && config.abi_path.is_some() {
      Err(DeploymentError::MissingBytecodePath(config.name.to_string()))
    } else if config.bytecode_path.is_some() && config.abi_path.is_some() {
      let bytecode_path = self.config.project_path.join(&PathBuf::from(config.bytecode_path.as_ref().unwrap()));
      let abi_path = self.config.project_path.join(&PathBuf::from(config.abi_path.as_ref().unwrap()));
      info!("Using pre-defined artifacts: {:?} and {:?}", &abi_path, &bytecode_path);
      Ok(Some((bytecode_path, abi_path)))
    } else {
      let artifacts_path = self.config.project_path.join(artifacts_path);
      let artifacts_dir = std::fs::read_dir(&artifacts_path)?;
      let artifact_names: Vec<PathBuf> = artifacts_dir.map(|res| res.unwrap().path()).collect();
      let smart_contract_name = config.instance_of.as_ref().unwrap_or(&config.name);

      if let Some(artifact) = artifact_names.iter().find(|path| path.to_string_lossy().to_string().contains(smart_contract_name)) {
        let file_extension = &artifact.extension().unwrap().to_str().unwrap();

        if file_extension == &ARTIFACT_EXTENSION_BINARY || file_extension == &ARTIFACT_EXTENSION_ABI {

          let file_bin_path = PathBuf::from(&artifact).with_extension(&ARTIFACT_EXTENSION_BINARY);
          let file_abi_path = PathBuf::from(&artifact).with_extension(&ARTIFACT_EXTENSION_ABI);

          if file_extension == &ARTIFACT_EXTENSION_BINARY && !file_abi_path.exists() {
            return Err(DeploymentError::MissingArtifact(ARTIFACT_EXTENSION_ABI.to_string(), file_bin_path.to_string_lossy().to_string()));
          } else if file_extension == &ARTIFACT_EXTENSION_ABI && !file_bin_path.exists() {
            return Err(DeploymentError::MissingArtifact(ARTIFACT_EXTENSION_BINARY.to_string(), file_abi_path.to_string_lossy().to_string()));
          }
          return Ok(Some((file_bin_path, file_abi_path)));
        }
      }
      Ok(None)
    }
  }

  fn get_first_block_hash(&self) -> Result<H256, DeploymentError> {
    let block = self.connector.get_first_block()?.unwrap();
    Ok(block.hash.unwrap())
  }
}

fn tokenize_args(args: &[SmartContractArg], deployed_contracts: &HashMap<Address, (String, Address, String, bool)>) -> Result<Vec<Token>, DeploymentError> {
  let mut tokenized_args: Vec<Token> = vec![];

  for arg in args {
    let param_type = Reader::read(&arg.kind).map_err(DeploymentError::InvalidParamType)?;

    let value = if ParamType::Address == param_type {
      if arg.value.starts_with('$') {
        let contract = deployed_contracts.values().find(|values| values.0 == arg.value[1..]).unwrap();
        format!("{:?}", &contract.1)[2..].to_owned()
      } else {
        arg.value[2..].to_owned()
      }
    } else {
      arg.value.to_owned()
    };

    let token = LenientTokenizer::tokenize(&param_type, &value).map_err(|err| DeploymentError::TokenizeParam(err, value.to_owned()))?;
    tokenized_args.push(token);
  }

  Ok(tokenized_args)
}

fn sort_by_dependencies(smart_contracts: &[SmartContractConfig]) -> Result<Vec<&SmartContractConfig>, DeploymentError> {
  let graph = DiGraphMap::<&str, ()>::from_edges(
    smart_contracts.iter().filter(|contract| contract.args.is_some()).flat_map(|contract| {
      contract.args.as_ref().unwrap().iter()
        .filter(|dep| dep.value.starts_with('$') && dep.kind == "address")
        .map(move |dep| (contract.name.as_str(), &dep.value[1..]))
    })
  ).into_graph::<u32>();

  let sorted_names = toposort(&graph, None).map_err(|err| DeploymentError::CyclicDependency(graph[err.node_id()].to_string()))?;
  let mut smart_contract_map = HashMap::new();

  for smart_contract_config in smart_contracts.iter() {
    smart_contract_map.insert(smart_contract_config.name.as_str(), smart_contract_config);
  }
  
  let mut sorted_smart_contracts = vec![];

  for i in sorted_names.into_iter() {
    if smart_contract_map.get(graph[i]).is_none() {
      return Err(DeploymentError::MissingConfigForReference(graph[i].to_owned()));
    }
    sorted_smart_contracts.push(smart_contract_map.remove(graph[i]).unwrap());
  }

  for (_name, config) in smart_contract_map {
    sorted_smart_contracts.push(config);
  }

  Ok(sorted_smart_contracts.into_iter().rev().collect())
}

#[cfg(test)]
mod tests {

  mod sort_by_dependencies {

    use super::super::sort_by_dependencies;
    use crate::config::ProjectConfig;

    fn project_config_from_string(config: &str) -> Result<ProjectConfig, toml::de::Error> {
      toml::from_str(&config)
    }

    #[test]
    fn it_should_fail_if_there_is_a_circular_dependency() {
      let project_config = project_config_from_string("
        [sources]
          artifacts = \"artifacts\"
          smart_contracts = [\"contracts/*.sol\"]
        [[deployment.smart_contracts]]
          name = \"Other\"
          args = [
            { value = \"$YetAnother\", kind = \"address\" }
          ]
        [[deployment.smart_contracts]]
          name = \"YetAnother\"
          args = [
            { value = \"$Other\", kind = \"address\" }
          ]
      ").unwrap();

      let smart_contracts = project_config.deployment.unwrap().smart_contracts;
      assert_eq!(sort_by_dependencies(&smart_contracts).is_err(), true);
    }

    #[test]
    fn it_should_fail_when_non_existing_smart_contract_is_referenced() {
      let project_config = project_config_from_string("
        [sources]
          artifacts = \"artifacts\"
          smart_contracts = [\"contracts/*.sol\"]
        [[deployment.smart_contracts]]
          name = \"A\"
          args = [
            { value = \"$B\", kind = \"address\" }
          ]
        [[deployment.smart_contracts]]
          name = \"B\"
          args = [
            { value = \"$C\", kind = \"address\" }
          ]
      ").unwrap();

      let smart_contracts = project_config.deployment.unwrap().smart_contracts;
      assert_eq!(sort_by_dependencies(&smart_contracts).is_err(), true);
    }

    #[test]
    fn it_should_sort_smart_contract_config_in_the_right_order() {
      let project_config = project_config_from_string("
        [sources]
          artifacts = \"artifacts\"
          smart_contracts = [\"contracts/*.sol\"]
        [[deployment.smart_contracts]]
          name = \"A\"
          args = [
            { value = \"$B\", kind = \"address\" },
            { value = \"$C\", kind = \"address\" },
            { value = \"$D\", kind = \"address\" },
          ]
        [[deployment.smart_contracts]]
          name = \"B\"
          args = [
            { value = \"$D\", kind = \"address\" }
          ]
        [[deployment.smart_contracts]]
          name = \"C\"
          args = [
            { value = \"$D\", kind = \"address\" }
          ]
        [[deployment.smart_contracts]]
          name = \"D\"
      ").unwrap();

      let smart_contracts = project_config.deployment.unwrap().smart_contracts;

      let sorted = sort_by_dependencies(&smart_contracts).unwrap();
      let expected = vec!["D", "B", "C", "A"];

      assert_eq!(sorted.iter().map(|contract| contract.name.as_str()).collect::<Vec<&str>>(), expected);
    }
  }
}
