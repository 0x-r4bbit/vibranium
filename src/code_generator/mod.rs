use std::path::{PathBuf};
use std::fs;

pub struct CodeGenerator;

const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";
const DEFAULT_CONTRACTS_DIRECTORY: &str = "contracts";

impl CodeGenerator {
  pub fn new() -> CodeGenerator {
    CodeGenerator
  }

  pub fn generate_project(&self, project_path: PathBuf) -> Result<(), std::io::Error> {
    for directory in vec![VIBRANIUM_PROJECT_DIRECTORY, DEFAULT_CONTRACTS_DIRECTORY] {
      let path = project_path.join(directory);
      if !path.exists() {
        fs::create_dir_all(path)?;
      }
    }
    Ok(())
  }
}
