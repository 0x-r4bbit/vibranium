extern crate assert_cmd;
extern crate predicates;
extern crate vibranium;
extern crate toml;
extern crate tempfile;

use std::process::Command;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{PathBuf};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir} ;
use vibranium::config::{ProjectConfig, ProjectSourcesConfig, ProjectCompilerConfig};

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

fn read_config(project_path: &PathBuf) -> Result<ProjectConfig, Box<std::error::Error>> {
  let project_config = toml::from_str(&fs::read_to_string(&project_path.join("vibranium.toml"))?)?;
  Ok(project_config)
}

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

  File::create(vibranium_dir.join("file1"))?;
  File::create(vibranium_dir.join("file2"))?;
  File::create(artifacts_dir.join("file1"))?;
  File::create(artifacts_dir.join("file2"))?;

  assert_eq!(fs::read_dir(&vibranium_dir).unwrap().count(), 2);
  assert_eq!(fs::read_dir(&artifacts_dir).unwrap().count(), 2);

  let mut cmd = Command::main_binary()?;
  cmd.arg("reset")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

  assert_eq!(fs::read_dir(&vibranium_dir).unwrap().count(), 0);
  assert_eq!(fs::read_dir(&artifacts_dir).unwrap().count(), 0);
  
  tmp_dir.close()?;
  Ok(())
}

#[test]
fn it_should_honor_changes_in_vibranium_toml_when_resetting_project() -> Result<(), Box<std::error::Error>> {

  let (tmp_dir, project_path) = setup_vibranium_project(None)?;
  let config_path = project_path.join("vibranium.toml");
  let updated_artifacts_dir: &str = "test_artifacts";

  let mut config: ProjectConfig = toml::from_str(&fs::read_to_string(&config_path)?)?;
  config.sources.artifacts = updated_artifacts_dir.to_string();
  let updated_config = toml::to_string(&config)?;
  fs::write(config_path, updated_config)?;

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
fn it_should_update_vibranium_config_file_via_config_command() -> Result<(), Box<std::error::Error>> {

  let (tmp_dir, project_path) = setup_vibranium_project(None)?;

  let mut cmd = Command::main_binary()?;

  cmd.arg("config")
      .arg("sources.artifacts")
      .arg("foo")
      .arg("--path")
      .arg(&project_path);

  cmd.assert().success();

  let config = read_config(&project_path)?;
  assert_eq!(config.sources.artifacts, "foo");

  tmp_dir.close()?;
  Ok(())
}

#[test]
fn it_accept_multi_value_config_options_using_array_syntax() -> Result<(), Box<std::error::Error>> {

  let (tmp_dir, project_path) = setup_vibranium_project(None)?;

  let mut cmd = Command::main_binary()?;

  cmd.arg("config")
      .arg("compiler.options")
      .arg("[--foo, --bar, --bazinga]")
      .arg("--path")
      .arg(&project_path);

  cmd.assert().success();

  let config = read_config(&project_path)?;
  assert_eq!(config.compiler.unwrap().options.unwrap(), ["--foo", "--bar", "--bazinga"]);

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

  let mut cmd = Command::main_binary()?;

  cmd.arg("config")
      .arg("unknown")
      .arg("foo")
      .arg("--path")
      .arg(&project_path);

  cmd.assert().success();

  tmp_dir.close()?;
  Ok(())
}

#[test]
fn it_should_fail_when_given_compiler_option_is_not_supported_and_no_compiler_options_specificed() -> Result<(), Box<std::error::Error>> {

  let (tmp_dir, project_path) = setup_vibranium_project(None)?;

  let mut cmd = Command::main_binary()?;

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

  // Vibranium won't even try to execute a compiler that it doesn't support
  // unless a users specifies all the needed options. That's why we overwrite
  // the config to use compiler.options as well as compiler.cmd.
  let config = ProjectConfig {
    sources: ProjectSourcesConfig {
      artifacts: "artifacts".to_string(),
      smart_contracts: vec!["contracts/*.sol".to_string()],
    },
    compiler: Some(ProjectCompilerConfig {
      cmd: Some("unsupported".to_string()),
      options: Some(vec!["--some-option".to_string()])
    })
  };

  let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

  let mut cmd = Command::main_binary()?;

  cmd.arg("compile")
      .arg("--path")
      .arg(&project_path);

  cmd.assert()
      .failure()
      .stderr(predicate::str::contains("Couldn't find executable for requested compiler"));

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

  // We don't provide any source files to solcjs, so we know it
  // will fail with the error message below.
  cmd.assert()
      .failure()
      .stderr(predicate::str::contains("Must provide a file"));

  tmp_dir.close()?;
  Ok(())
}

#[test]
fn it_should_honor_compiler_options_specified_in_config_file() -> Result<(), Box<std::error::Error>> {

  // We overwrite the default configuration to set the compiler
  // to `solcjs`. Vibranium uses `solc` as default.
  let config = ProjectConfig {
    sources: ProjectSourcesConfig {
      artifacts: "artifacts".to_string(),
      smart_contracts: vec!["contracts/*.sol".to_string()],
    },
    compiler: Some(ProjectCompilerConfig {
      cmd: Some("solcjs".to_string()),
      options: None
    })
  };

  let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

  let mut cmd = Command::main_binary()?;

  // There are no Smart Contract files in the generated project
  // so if everything goes as expected, this command fails with
  // `solcjs` exiting with the error message below.
  cmd.arg("compile")
      .arg("--path")
      .arg(&project_path);

  cmd.assert()
      .failure()
      .stderr(predicate::str::contains("Must provide a file"));
  
  tmp_dir.close()?;
  Ok(())
}

#[test]
fn it_should_override_config_file_compiler_options_with_cli_options() -> Result<(), Box<std::error::Error>> {

  let config = ProjectConfig {
    sources: ProjectSourcesConfig {
      artifacts: "artifacts".to_string(),
      smart_contracts: vec!["contracts/*.sol".to_string()],
    },
    compiler: Some(ProjectCompilerConfig {
      cmd: Some("ignored".to_string()),
      options: None
    })
  };

  let (tmp_dir, project_path) = setup_vibranium_project(Some(config))?;

  let mut cmd = Command::main_binary()?;

  cmd.arg("compile")
      .arg("--compiler")
      .arg("solcjs")
      .arg("--path")
      .arg(&project_path);

  // Failure is the expected behaviour here as we don't provide any Smart Contract
  // source files to `solcjs`.
  cmd.assert()
      .failure()
      .stderr(predicate::str::contains("Must provide a file"));

  tmp_dir.close()?;
  Ok(())
}
