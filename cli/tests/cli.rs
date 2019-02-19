extern crate assert_cmd;
extern crate predicates;
extern crate vibranium;
extern crate toml;

use std::process::Command;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use vibranium::project_generator::ProjectConfig;

const TMP_TEST_DIR: &str = "/tmp";

fn setup_vibranium_project(directory: &str) -> Result<PathBuf, Box<std::error::Error>> {
  let _ = fs::create_dir(TMP_TEST_DIR);
  let project_path = Path::new(TMP_TEST_DIR).join(directory);
  fs::create_dir_all(&project_path)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("init")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();
  Ok(project_path)
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
  let project_path = setup_vibranium_project("vibranium_test_dapp__1")?;

  assert_eq!(project_path.join(".vibranium").exists(), true);
  assert_eq!(project_path.join("artifacts").exists(), true);
  assert_eq!(project_path.join("contracts").exists(), true);
  assert_eq!(project_path.join("vibranium.toml").is_file(), true);

  let _ = fs::remove_dir_all(project_path);
  Ok(())
}

#[test]
fn it_should_fail_on_reset_if_project_is_not_a_vibranium_project() -> Result<(), Box<std::error::Error>> {
  let _ = fs::create_dir(TMP_TEST_DIR);
  let project_path = Path::new(TMP_TEST_DIR).join("vibranium_test_dapp__2");
  fs::create_dir_all(&project_path)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("reset")
      .arg("--path")
      .arg(&project_path);
  cmd.assert()
      .failure()
      .stderr(predicate::str::contains("Not a Vibranium project"));

  let _ = fs::remove_dir_all(project_path);
  Ok(())
}

#[test]
fn it_should_reset_project() -> Result<(), Box<std::error::Error>> {

  let project_path = setup_vibranium_project("vibranium_test_dapp__3")?;
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
  
  let _ = fs::remove_dir_all(project_path);
  Ok(())
}

#[test]
fn it_should_honor_changes_in_vibranium_toml_when_resetting_project() -> Result<(), Box<std::error::Error>> {

  let project_path = setup_vibranium_project("vibranium_test_dapp__4")?;
  let config_path = project_path.join("vibranium.toml");
  let updated_artifacts_dir: &str = "test_artifacts";

  let mut config: ProjectConfig = toml::from_str(&fs::read_to_string(&config_path)?)?;
  config.artifacts_dir = updated_artifacts_dir.to_string();
  let updated_config = toml::to_string(&config)?;
  fs::write(config_path, updated_config)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("reset")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

  assert_eq!(project_path.join(updated_artifacts_dir).exists(), true);

  let _ = fs::remove_dir_all(project_path);
  Ok(())
}
