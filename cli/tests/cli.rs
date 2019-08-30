extern crate assert_cmd;
extern crate predicates;
extern crate vibranium;
extern crate toml;
extern crate tempfile;

use std::process::Command;
use assert_cmd::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{PathBuf};
use tempfile::{tempdir, TempDir} ;
use vibranium::config::ProjectConfig;

fn setup_vibranium_project(config: Option<ProjectConfig>) -> Result<(TempDir, PathBuf), Box<std::error::Error>> {
  let tmp_dir = tempdir()?;
  let project_path = tmp_dir.path().join("test_dapp");
  let _ = fs::create_dir(&project_path);

  let mut cmd = Command::main_binary()?;
  cmd.arg("init")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

  if let Some(cfg) = config {
    let config_toml = toml::to_string(&cfg).unwrap();

    let mut config_file = OpenOptions::new()
      .write(true)
      .open(&project_path.join("vibranium.toml")).unwrap();
    config_file.write_all(config_toml.as_bytes()).unwrap();
  }

  Ok((tmp_dir, project_path))
}

fn create_test_file(project_path: &PathBuf, dest: &str, name: &str) -> Result<(), Box<std::error::Error>> {
  let cwd = std::env::current_dir()?;
  let fixture_file = cwd.join("tests").join("fixtures").join(name);
  let test_file = project_path.join(dest).join(name);
  std::fs::copy(fixture_file, test_file)?;
  Ok(())
}

fn create_test_contract(project_path: &PathBuf, name: &str) -> Result<(), Box<std::error::Error>> {
  create_test_file(project_path, "contracts", name)
}

fn create_test_artifact(project_path: &PathBuf, name: &str) -> Result<(), Box<std::error::Error>> {
  create_test_file(project_path, "artifacts", name)
}

fn read_config(project_path: &PathBuf) -> Result<ProjectConfig, Box<std::error::Error>> {
  let project_config = toml::from_str(&fs::read_to_string(&project_path.join("vibranium.toml"))?)?;
  Ok(project_config)
}

fn set_configurations(configs: Vec<(&str, &str)>, project_path: &PathBuf) -> Result<(), Box<std::error::Error>> {
  for (config, value) in configs {
    set_configuration(&config, &value, &project_path)?;
  }
  Ok(())
}

fn set_configuration(config: &str, value: &str, project_path: &PathBuf) -> Result<(), Box<std::error::Error>> {
  let mut cmd = Command::main_binary()?;
  cmd.arg("config")
      .arg(config)
      .arg(value)
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();
  Ok(())
}

#[cfg(test)]
mod init_cmd {

  use std::process::Command;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;
  
  use super::setup_vibranium_project;
  use super::read_config;

  #[test]
  fn it_should_fail_on_init_if_project_path_doesnt_exist() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("init")
        .arg("--path")
        .arg("/tmp/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't find directory for given project path"));
    Ok(())
  }

  #[test]
  fn it_should_initialize_project() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    assert_eq!(project_path.join(".vibranium").exists(), true);
    assert_eq!(project_path.join("artifacts").exists(), true);
    assert_eq!(project_path.join("contracts").exists(), true);
    assert_eq!(project_path.join("vibranium.toml").is_file(), true);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_initialize_project_with_default_config_preset() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let config = read_config(&project_path)?;
    assert_eq!(config.sources.artifacts, "artifacts");
    assert_eq!(config.sources.smart_contracts, vec!["contracts/*.sol"]);

    let compiler_config = config.compiler.unwrap();
    let compiler_options = compiler_config.options.unwrap();

    assert_eq!(&compiler_config.cmd.unwrap(), "solc");
    assert_eq!(&compiler_options[0], "--abi");
    assert_eq!(&compiler_options[1], "--bin");
    assert_eq!(&compiler_options[2], "--overwrite");

    let blockchain_config = config.blockchain.unwrap();
    let blockchain_options = blockchain_config.options;

    assert_eq!(&blockchain_config.cmd.unwrap(), "parity");
    assert_eq!(blockchain_options.is_none(), true);

    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod reset_cmd {

  use std::process::Command;
  use std::fs::{self, File};
  use assert_cmd::prelude::*;
  use predicates::prelude::*;
  use tempfile::tempdir;
  use vibranium::config::{
    ProjectConfig,
    ProjectDeploymentConfig,
    SmartContractConfig,
    SmartContractArg
  };

  use super::create_test_artifact;
  use super::setup_vibranium_project;
  use super::set_configuration;
  use super::set_configurations;
  use super::read_config;

  #[test]
  fn it_should_fail_on_reset_if_project_is_not_a_vibranium_project() -> Result<(), Box<std::error::Error>> {
    let tmp_dir = tempdir()?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("reset")
        .arg("--path")
        .arg(&tmp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not a Vibranium project"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_reset_project() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    let vibranium_dir = project_path.join(".vibranium");
    let artifacts_dir = project_path.join("artifacts");

    assert_eq!(vibranium_dir.exists(), true);
    assert_eq!(artifacts_dir.exists(), true);

    File::create(artifacts_dir.join("file1"))?;
    File::create(artifacts_dir.join("file2"))?;

    assert_eq!(fs::read_dir(&artifacts_dir).unwrap().count(), 2);

    let mut cmd = Command::main_binary()?;
    cmd.arg("reset")
        .arg("--path")
        .arg(&project_path);
    cmd.assert().success();

    assert_eq!(fs::read_dir(&artifacts_dir).unwrap().count(), 0);
    
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_honor_changes_in_vibranium_toml_when_resetting_project() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    let updated_artifacts_dir: &str = "test_artifacts";

    set_configuration("sources.artifacts", &updated_artifacts_dir, &project_path)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("reset")
        .arg("--path")
        .arg(&project_path);
    cmd.assert().success();

    assert_eq!(project_path.join(updated_artifacts_dir).exists(), true);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_restore_config_file_with_defaults_when_resetting_project() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    set_configurations(vec![
      ("sources.artifacts", "something-else"),
      ("sources.smart_contracts", "[foo, bar]"),
      ("compiler.cmd", "something-else"),
      ("blockchain.cmd", "something-else"),
    ], &project_path)?;

    let config = read_config(&project_path)?;

    assert_eq!(config.sources.artifacts, "something-else");
    assert_eq!(config.sources.smart_contracts, vec!["foo", "bar"]);
    assert_eq!(config.compiler.unwrap().cmd.unwrap(), "something-else");
    assert_eq!(config.blockchain.unwrap().cmd.unwrap(), "something-else");

    let mut cmd = Command::main_binary()?;
    cmd.arg("reset")
        .arg("--restore-config")
        .arg("--path")
        .arg(&project_path);
    cmd.assert().success();

    let default_config = ProjectConfig::default();
    let config = read_config(&project_path)?;

    assert_eq!(config.sources.artifacts, default_config.sources.artifacts);
    assert_eq!(config.sources.smart_contracts, default_config.sources.smart_contracts);
    assert_eq!(config.compiler.unwrap().cmd, default_config.compiler.unwrap().cmd);
    assert_eq!(config.blockchain.unwrap().cmd, default_config.blockchain.unwrap().cmd);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_reset_only_tracking_data_if_flag_is_provided() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    let vibranium_dir = project_path.join(".vibranium");
    let tracking_file = project_path.join(".vibranium").join("tracking.toml");
    assert_eq!(tracking_file.exists(), true);

    let mut cmd = Command::main_binary()?;
    cmd.arg("reset")
        .arg("--path")
        .arg(&project_path)
        .arg("--tracking-data");

    cmd.assert().success();

    assert_eq!(vibranium_dir.exists(), true);
    assert_eq!(tracking_file.exists(), false);

    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod config_cmd {

  use std::process::Command;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;

  use super::setup_vibranium_project;
  use super::set_configuration;
  use super::read_config;

  #[test]
  fn it_should_update_vibranium_config_file_via_config_command() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    set_configuration("sources.artifacts", "foo", &project_path)?;

    let config = read_config(&project_path)?;
    assert_eq!(config.sources.artifacts, "foo");

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_accept_multi_value_config_options_using_array_syntax() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    set_configuration("compiler.options", "[--foo, --bar, --bazinga]", &project_path)?;

    let config = read_config(&project_path)?;
    assert_eq!(config.compiler.unwrap().options.unwrap(), ["--foo", "--bar", "--bazinga"]);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_remove_empty_values_when_setting_multi_value_options() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    set_configuration("compiler.options", "[foo, ]", &project_path)?;

    let config = read_config(&project_path)?;
    assert_eq!(config.compiler.unwrap().options.unwrap(), ["foo"]);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_when_setting_incompatible_config_value_for_config_option() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;

    cmd.arg("config")
        .arg("compiler.options")
        .arg("single-value")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't deserialize vibranium config"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_ignore_config_options_that_do_not_exist() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    set_configuration("unknown", "foo", &project_path)?;
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_remove_config_option() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    set_configuration("compiler.cmd", "foo", &project_path)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("config")
        .arg("--unset")
        .arg("compiler.cmd")
        .arg("--path")
        .arg(&project_path);
    cmd.assert().success();

    let config = read_config(&project_path)?;
    assert_eq!(config.compiler.unwrap().cmd, None);
    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod compile_cmd {

  use std::fs;
  use std::process::Command;
  use std::path::PathBuf;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;
  use vibranium::config::ProjectConfig;

  use super::setup_vibranium_project;
  use super::create_test_contract;
  use super::set_configuration;
  use super::set_configurations;

  #[test]
  fn it_should_fail_when_given_compiler_option_is_not_supported_and_no_compiler_options_specificed() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;

    set_configuration("compiler.options", "[]", &project_path)?;

    cmd.arg("compile")
        .arg("--compiler")
        .arg("unsupported")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No built-in support for requested compiler"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_when_given_compiler_is_not_installed() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    // Vibranium won't even try to execute a compiler that it doesn't support
    // unless a users specifies all the needed options. That's why we overwrite
    // the config to use compiler.options as well as compiler.cmd.
    set_configurations(vec![
      ("compiler.cmd", "unsupported"),
      ("compiler.options", "[--some-option]"),
    ], &project_path)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().failure();

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_when_compiler_program_fails() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;

    cmd.arg("compile")
        .arg("--compiler")
        .arg("solcjs")
        .arg("--path")
        .arg(&project_path);

    // We don't provide any source files to solc, so we know it will
    cmd.assert().failure();
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_honor_compiler_options_specified_in_config_file() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    // We overwrite the default configuration to set the compiler
    // to `solcjs`. Vibranium uses `solc` as default.
    set_configuration("compiler.cmd", "solcjs", &project_path)?;

    let mut cmd = Command::main_binary()?;

    cmd.arg("compile")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().failure();
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_override_config_file_compiler_options_with_cli_options() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    set_configuration("compiler.cmd", "ignored", &project_path)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--compiler")
        .arg("something")
        .arg("--path")
        .arg(&project_path);

    // Failure is the expected behaviour here as we don't provide a valid 
    // compiler option
    cmd.assert().failure();
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_transform_source_imports_when_using_solidity() -> Result<(), Box<std::error::Error>> {

    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    let node_modules_path = project_path.join("node_modules");

    let import_path = PathBuf::from("@some-package").join("contracts").join("something.sol");
    let absolute_path = node_modules_path.join(&import_path);

    fs::create_dir_all(absolute_path.parent().unwrap())?;
    fs::File::create(&absolute_path)?;

    create_test_contract(&project_path, "test_contract_with_node_module_import.sol")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--compiler")
        .arg("solcjs")
        .arg("--path")
        .arg(&project_path)
        .arg("--verbose");

    // If this command succeeds we already know transformation has worked as the path used
    // inside the test contract file doesn't exist otherwise.
    cmd.assert().success();

    assert_eq!(project_path.join(".vibranium").join("contracts").join(&absolute_path).exists(), true);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_turn_off_smart_imports_when_flag_is_applied() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;
    create_test_contract(&project_path, "simple_test_contract.sol")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--compiler")
        .arg("solcjs")
        .arg("--path")
        .arg(&project_path)
        .arg("--no-smart-imports")
        .arg("--verbose");

    cmd.assert().success();

    assert_eq!(project_path.join(".vibranium").join("contracts").exists(), false);

    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod accounts_cmd {

  use std::process::Command;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;

  use super::setup_vibranium_project;

  #[test]
  fn it_should_output_local_blockchains_dev_accounts() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("accounts")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
      .success()
        .stdout(predicate::str::contains("(0) 0x"));

    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod deploy_cmd {

  use std::process::Command;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;

  use super::setup_vibranium_project;
  use super::set_configuration;
  use super::create_test_contract;
  use super::create_test_artifact;
  use vibranium::config::{
    ProjectConfig,
    ProjectDeploymentConfig,
    SmartContractConfig,
    SmartContractArg
  };

  #[test]
  fn it_should_fail_if_no_deployment_config_is_provided() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't find deployment configuration"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_skip_deployment_if_no_artifacts_exist() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      smart_contracts: vec![],
      tracking_enabled: None,
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Nothing to deploy"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_skip_deployment_if_address_is_provided_in_configuration() -> Result<(), Box<std::error::Error>> {
    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: Some("0x552C51e32c70D5859E5163D319531B63e5dbBFF7".to_string()),
          instance_of: None,
          args: None,
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        }
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("skipped"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_if_provided_address_is_invalid() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: Some("0x".to_string()),
          instance_of: None,
          args: None,
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        }
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid address"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_if_parameter_args_are_not_valid() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg {
              value: "2".to_string(),
              kind: "invalid".to_string()
            }
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        }
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't read Smart Contract constructor parameter"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_if_it_can_not_tokenize_args() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg {
              value: "200".to_string(),
              kind: "bool".to_string()
            }
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        }
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't tokenize Smart Contract constructor parameter"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_fail_if_artifacts_are_partially_missing() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![SmartContractConfig {
        name: contract_name.to_string(),
        address: None,
        instance_of: None,
        args: None,
        gas_limit: None,
        gas_price: None,
        abi_path: None,
        bytecode_path: None,
      }],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;
    // Having a `contract.bin` but no `contract.abi` will cause Vibranium
    // to stop the deployment.
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Couldn't find abi file for"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_use_gas_limit_defined_in_smart_contract_config() -> Result<(), Box<std::error::Error>> {
    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![SmartContractConfig {
        name: contract_name.to_string(),
        address: None,
        instance_of: None,
        args: None,
        gas_limit: Some(20000),
        gas_price: None,
        abi_path: None,
        bytecode_path: None,
      }],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    // The gas limit for SimpleTestContract is too low,
    // so we expect the deployment to fail.
    cmd.assert().failure();
;
    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_deploy_smart_contracts() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_deploy_multiple_smart_contracts() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";
    let contract_name_2 = "SimpleTestContract2";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
        SmartContractConfig {
          name: contract_name_2.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;
    create_test_contract(&project_path, "simple_test_contract.sol")?;
    create_test_contract(&project_path, "simple_test_contract_2.sol")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--compiler")
        .arg("solcjs")
        .arg("--path")
        .arg(&project_path)
        .arg("--verbose");

    cmd.assert().success();
    
    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_track_deployed_smart_contracts() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    let tracking_file = project_path.join(".vibranium").join("tracking.toml");
    assert_eq!(tracking_file.exists(), true);

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
       .success()
       .stdout(predicate::str::contains("(skipped)"));

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_not_track_deployment_when_tracking_is_turned_off() -> Result<(), Box<std::error::Error>> {
    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: Some(false),
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    let tracking_file = project_path.join(".vibranium").join("tracking.toml");
    assert_eq!(tracking_file.exists(), false);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_not_track_deployment_when_notracking_flag_is_set() -> Result<(), Box<std::error::Error>> {
    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--no-tracking")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    let tracking_file = project_path.join(".vibranium").join("tracking.toml");
    assert_eq!(tracking_file.exists(), false);

    tmp_dir.close()?;
    Ok(())
  }

  #[test]
  fn it_should_use_bytecode_of_specified_instance_of_configuration() -> Result<(), Box<std::error::Error>> {
    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: "InstanceOfSimpleStorage".to_string(),
          address: None,
          args: Some(vec![
            SmartContractArg { value: "500".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          // we update this value manually down below due to toml-rs'
          // ValueAfterTable error.
          instance_of: None,
          abi_path: None,
          bytecode_path: None,
        }
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;
    create_test_contract(&project_path, "simple_test_contract.sol")?;

    set_configuration("deployment.smart_contracts[1].instance_of", &contract_name.to_string(), &project_path)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("compile")
        .arg("--compiler")
        .arg("solcjs")
        .arg("--path")
        .arg(&project_path)
        .arg("--verbose");

    cmd.assert().success();

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    tmp_dir.close()?;
    Ok(())
  }
}

#[cfg(test)]
mod list_cmd {

  use std::process::Command;
  use assert_cmd::prelude::*;
  use predicates::prelude::*;

  use super::setup_vibranium_project;
  use super::create_test_artifact;
  use vibranium::config::{
    ProjectConfig,
    ProjectDeploymentConfig,
    SmartContractConfig,
    SmartContractArg
  };

  #[test]
  fn it_should_show_no_tracking_data_exists_if_no_tracking_database() -> Result<(), Box<std::error::Error>> {
    let (tmp_dir, project_path) = setup_vibranium_project(None)?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("list")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No Smart Contract data"));

    tmp_dir.close()?;
    Ok(())
  }
  
  #[test]
  fn it_should_show_tracked_smart_contract_data() -> Result<(), Box<std::error::Error>> {

    let mut config = ProjectConfig::default();
    let contract_name = "SimpleTestContract";

    config.deployment = Some(ProjectDeploymentConfig {
      gas_limit: None,
      gas_price: None,
      tx_confirmations: None,
      tracking_enabled: None,
      smart_contracts: vec![
        SmartContractConfig {
          name: contract_name.to_string(),
          address: None,
          instance_of: None,
          args: Some(vec![
            SmartContractArg { value: "200".to_string(),kind: "uint".to_string() },
          ]),
          gas_limit: None,
          gas_price: None,
          abi_path: None,
          bytecode_path: None,
        },
      ],
      accounts: None,
    });

    let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

    create_test_artifact(&project_path, "SimpleTestContract.abi")?;
    create_test_artifact(&project_path, "SimpleTestContract.bin")?;

    let mut cmd = Command::main_binary()?;
    cmd.arg("deploy")
        .arg("--path")
        .arg(&project_path);

    cmd.assert().success();

    let mut cmd = Command::main_binary()?;
    cmd.arg("list")
        .arg("--path")
        .arg(&project_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Deployed Smart Contracts:\n  0x"));

    tmp_dir.close()?;
    Ok(())
  }
}
