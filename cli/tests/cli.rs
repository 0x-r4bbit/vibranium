extern crate assert_cmd;
extern crate predicates;

use std::process::Command;
use std::fs;
use std::path::Path;
use assert_cmd::prelude::*;
use predicates::prelude::*;

const TMP_TEST_DIR: &str = "/tmp";

#[test]
fn project_path_doesnt_exist() -> Result<(), Box<std::error::Error>> {
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
fn init_project() -> Result<(), Box<std::error::Error>> {
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
fn not_a_vibranium_project() -> Result<(), Box<std::error::Error>> {
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
fn reset_project() -> Result<(), Box<std::error::Error>> {
  let _ = fs::create_dir(TMP_TEST_DIR);
  let project_path = Path::new(TMP_TEST_DIR).join("vibranium_test_dapp__3");
  fs::create_dir_all(&project_path)?;

  let mut cmd = Command::main_binary()?;
  cmd.arg("init")
      .arg("--path")
      .arg(&project_path);
  cmd.assert().success();

  assert_eq!(project_path.join(".vibranium").exists(), true);
  assert_eq!(fs::read_dir(project_path.join(".vibranium")).unwrap().count(), 0);
  assert_eq!(project_path.join("artifacts").exists(), true);
  assert_eq!(fs::read_dir(project_path.join("artifacts")).unwrap().count(), 0);
  
  let _ = fs::remove_dir_all(project_path);
  Ok(())
}
