use crate::utils as lib_utils;
use lib_utils::adjust_canonicalization;
use regex::Regex;
use std::path::{PathBuf, Path, Component};
use super::error;

const NODE_MODULES_DIR: &str = "node_modules";
pub const INTERNAL_SOURCE_DIR: &str = "contracts";

pub fn strip_absolute_prefix(path: &Path) -> PathBuf {
  path.components().filter(|c| *c != Component::RootDir).collect::<PathBuf>()
}

#[derive(PartialEq, Eq, Debug)]
pub enum ImportType {
  Internal,
  Node,
  External,
}

type ResolvedImport = (ImportType, PathBuf, PathBuf);

pub fn extract_imports(contents: &mut str) -> Vec<String> {
  let re = Regex::new(r#"import[\s]*(['"])(.*)(['"]);"#).unwrap();
  re.captures_iter(&contents).map(|x| x[2].to_string()).collect()
}

pub fn resolve_import(import: &str, file_path: &Path, project_path: &PathBuf, destination_path: &PathBuf) -> Result<ResolvedImport, error::CompilerError> {
  let mut import = PathBuf::from(&import).components().collect::<PathBuf>();

  if is_node_module_import(&import, file_path, &project_path) {
    let resolved_path = if import.starts_with("./") || import.starts_with("..") {
      adjust_canonicalization(file_path.join(&import).canonicalize().unwrap())
    } else {
      project_path.join(NODE_MODULES_DIR).join(&import)
    };
    let normalized_path = destination_path.join(resolved_path.strip_prefix(project_path).unwrap());
    Ok((ImportType::Node, resolved_path, normalized_path))
  } else if is_internal_import(&import, file_path, &project_path) {
    if !import.is_absolute() {
      import = file_path.join(import);
    }
    let resolved_path = adjust_canonicalization(import.canonicalize().unwrap());
    let normalized_path = destination_path.join(resolved_path.strip_prefix(project_path.join(INTERNAL_SOURCE_DIR)).unwrap());
    Ok((ImportType::Internal, resolved_path, normalized_path))
  } else {
    let canonicalized_import = import.canonicalize().map_err(|_| error::CompilerError::ImportError(file_path.join(import)))?;
    let resolved_path = adjust_canonicalization(canonicalized_import);
    let normalized_path = destination_path.join(strip_absolute_prefix(&resolved_path));
    Ok((ImportType::External, resolved_path, normalized_path))
  }
}

fn is_node_module_import<T: AsRef<Path>>(path: T, parent_path: &Path, project_path: &PathBuf) -> bool {
  if path.as_ref().is_absolute() {
    info!("Found absolute import: {:?}", &path.as_ref());
    path.as_ref().starts_with(project_path.join(NODE_MODULES_DIR)) && path.as_ref().exists()
  } else if path.as_ref().starts_with(".") || path.as_ref().starts_with("..") {
    info!("Found relative import: {:?}", &path.as_ref());
    match parent_path.join(path).canonicalize() {
      Ok(p) => p.starts_with(project_path.join(NODE_MODULES_DIR)),
      Err(_) => false
    }
  } else if !path.as_ref().starts_with(NODE_MODULES_DIR) {
    info!("Found implicit node_module import: {:?}", &path.as_ref());
    project_path.join(NODE_MODULES_DIR).join(&path).exists()
  } else {
    info!("Found explicit node_module import: {:?}", &path.as_ref());
    project_path.join(&path).exists()
  }
}

fn is_internal_import<T: AsRef<Path>>(path: T, parent_path: &Path, project_path: &PathBuf) -> bool {
  if !path.as_ref().is_absolute() {
    match parent_path.join(&path).canonicalize() {
      Ok(p) => adjust_canonicalization(p).starts_with(project_path),
      Err(_) => false
    }
  } else {
    path.as_ref().starts_with(project_path)
  }
}

pub fn get_destination_path(original_path: &PathBuf, project_path: &PathBuf, destination_root: &PathBuf) -> PathBuf {
  if let Ok(path) = original_path.strip_prefix(&project_path) {
    if path.starts_with(NODE_MODULES_DIR) {
      destination_root.join(path)
    } else {
      destination_root.join(path.strip_prefix(INTERNAL_SOURCE_DIR).unwrap())
    }
  } else {
    destination_root.join(strip_absolute_prefix(&original_path))
  }
}

#[cfg(test)]
mod tests {

  extern crate tempfile;

  use std::fs;
  use std::path::PathBuf;
  use tempfile::{tempdir, TempDir};
  use super::adjust_canonicalization;

  fn create_test_project() -> Result<(TempDir, PathBuf), Box<std::error::Error>> {
    let tmp_dir = tempdir()?;
    let mut project_path = tmp_dir.path().join("test_dapp");
    fs::create_dir(&project_path)?;
    project_path = adjust_canonicalization(project_path.canonicalize().unwrap());
    Ok((tmp_dir, project_path))
  }

  mod resolve_import {

    use std::fs;
    use std::path::PathBuf;
    use super::create_test_project;
    use super::super::ImportType;
    use super::super::resolve_import;
    use super::super::strip_absolute_prefix;

    #[test]
    fn it_should_resolve_internal_imports() -> Result<(), Box<std::error::Error>>  {
      let (tmp_dir, project_path) = create_test_project()?;
      let import_path = PathBuf::from("contracts").join("something").join("foo.sol");
      let absolute_path = project_path.join(&import_path);
      let destination_path = project_path.join(".vibranium").join("contracts");

      fs::create_dir_all(absolute_path.parent().unwrap())?;
      fs::File::create(&absolute_path)?;

      let resolved_import = resolve_import(&import_path.to_str().unwrap(), &project_path, &project_path, &destination_path)?;

      assert_eq!(resolved_import.0, ImportType::Internal);
      assert_eq!(resolved_import.1, project_path.join(&import_path));
      assert_eq!(resolved_import.2, destination_path.join(&import_path.strip_prefix("contracts").unwrap()));

      tmp_dir.close()?;
      Ok(())
    }

    #[test]
    fn it_should_resolve_node_module_imports() -> Result<(), Box<std::error::Error>>  {
      let (tmp_dir, project_path) = create_test_project()?;
      let import_path = PathBuf::from("@some").join("nodepackage").join("some.sol");
      let absolute_node_modules_path = project_path.join("node_modules");
      let absolute_path = absolute_node_modules_path.join(&import_path);
      let destination_path = project_path.join(".vibranium").join("contracts");

      fs::create_dir_all(absolute_path.parent().unwrap())?;
      fs::File::create(&absolute_path)?;

      let resolved_import = resolve_import(&import_path.to_str().unwrap(), &absolute_node_modules_path, &project_path, &destination_path)?;

      assert_eq!(resolved_import.0, ImportType::Node);
      assert_eq!(resolved_import.1, absolute_node_modules_path.join(&import_path));
      assert_eq!(resolved_import.2, destination_path.join("node_modules").join(&import_path));

      tmp_dir.close()?;
      Ok(())
    }

    #[test]
    fn it_should_resolve_external_imports() -> Result<(), Box<std::error::Error>>  {
      let (tmp_dir, project_path) = create_test_project()?;
      let (tmp_dir2, external_project_path) = create_test_project()?;

      let destination_path = project_path.join(".vibranium").join("contracts");
      let absolute_external_path = external_project_path.join("contracts").join("something").join("foo.sol");

      fs::create_dir_all(absolute_external_path.parent().unwrap())?;
      fs::File::create(&absolute_external_path)?;

      let resolved_import = resolve_import(&absolute_external_path.to_str().unwrap(), &external_project_path, &project_path, &destination_path)?;

      assert_eq!(resolved_import.0, ImportType::External);
      assert_eq!(resolved_import.1, absolute_external_path);
      assert_eq!(resolved_import.2, destination_path.join(strip_absolute_prefix(&absolute_external_path)));

      tmp_dir.close()?;
      tmp_dir2.close()?;
      Ok(())
    }
  }

  mod is_node_module_import {

    use std::fs;
    use std::path::PathBuf;
    use super::create_test_project;
    use super::super::is_node_module_import;

    #[test]
    fn it_should_be_true_with_implicit_import_syntax() -> Result<(), Box<std::error::Error>> {
      let (tmp_dir, project_path) = create_test_project()?;

      let import_path = PathBuf::from("@aragon").join("something").join("foo.sol");
      let relative_node_module_path = PathBuf::from("node_modules").join(&import_path);
      let absolute_node_module_path = project_path.join(&relative_node_module_path);

      fs::create_dir_all(&absolute_node_module_path.parent().unwrap())?;
      fs::File::create(&absolute_node_module_path)?;

      assert_eq!(is_node_module_import(&import_path, &project_path, &project_path), true);

      tmp_dir.close()?;
      Ok(())
    }

    #[test]
    fn it_should_be_true_with_explicit_import_syntax() -> Result<(), Box<std::error::Error>> {
      let (tmp_dir, project_path) = create_test_project()?;
      let import_path = PathBuf::from("@aragon").join("something").join("foo.sol");
      let relative_node_module_path = PathBuf::from("node_modules").join(&import_path);
      let absolute_node_module_path = project_path.join(&relative_node_module_path);

      fs::create_dir_all(&absolute_node_module_path.parent().unwrap())?;
      fs::File::create(&absolute_node_module_path)?;

      assert_eq!(is_node_module_import(&relative_node_module_path, &project_path, &project_path), true);

      tmp_dir.close()?;
      Ok(())
    }
  }

  mod is_internal_import {

    use std::fs;
    use std::path::PathBuf;
    use super::create_test_project;
    use super::super::is_internal_import;

    #[test]
    fn it_should_be_true_if_absolute_import_is_within_project() -> Result<(), Box<std::error::Error>> {
      let (tmp_dir, project_path) = create_test_project()?;
      let import_path = PathBuf::from("contracts").join("something").join("foo.sol");
      let absolute_path = project_path.join(&import_path);

      fs::create_dir_all(&absolute_path.parent().unwrap())?;
      fs::File::create(&absolute_path)?;

      assert_eq!(is_internal_import(&absolute_path, &project_path, &project_path), true);

      tmp_dir.close()?;
      Ok(())
    }

    #[test]
    fn it_should_be_true_if_relative_import_is_within_project() -> Result<(), Box<std::error::Error>> {
      let (tmp_dir, project_path) = create_test_project()?;
      let import_path = PathBuf::from("contracts").join("something").join("foo.sol");
      let absolute_path = project_path.join(&import_path);

      fs::create_dir_all(&absolute_path.parent().unwrap())?;
      fs::File::create(&absolute_path)?;

      assert_eq!(is_internal_import(&import_path, &project_path, &project_path), true);

      tmp_dir.close()?;
      Ok(())
    }
  }
}
