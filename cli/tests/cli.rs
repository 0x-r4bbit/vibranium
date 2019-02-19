extern crate assert_cmd;
extern crate predicates;

use std::process::Command;
use std::fs::{self, File};
use std::path::Path;
use assert_cmd::prelude::*;
use predicates::prelude::*;

const TMP_TEST_DIR: &str = "/tmp";

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
  let _ = fs::create_dir(TMP_TEST_DIR);
  let project_path = Path::new(TMP_TEST_DIR).join("vibranium_test_dapp__1");
  fs::create_dir_all(&project_path)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("init")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

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

  let _ = fs::create_dir(TMP_TEST_DIR);
  let project_path = Path::new(TMP_TEST_DIR).join("vibranium_test_dapp__3");
  let vibranium_dir = project_path.join(".vibranium");
  let artifacts_dir = project_path.join("artifacts");

  fs::create_dir_all(&project_path)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("init")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

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
