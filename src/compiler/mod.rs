pub mod error;
pub mod support;
mod utils;

use crate::config;
use crate::utils as lib_utils;
use glob::glob;
use std::fs;
use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::io::Write;
use std::collections::HashSet;
use support::SupportedCompilers;
use utils::{INTERNAL_SOURCE_DIR};


#[derive(Debug)]
pub struct CompilerConfig {
  pub compiler: Option<String>,
  pub compiler_options: Option<Vec<String>>,
}

pub struct Compiler<'a> {
  config: &'a config::Config,
}

impl<'a> Compiler<'a> {
  pub fn new(config: &config::Config) -> Compiler {
    Compiler {
      config
    }
  }

  pub fn normalize_imports(&self) -> Result<Vec<PathBuf>, error::CompilerError> {
    let project_config = self.config.read()?;
    let destination_root = self.config.vibranium_dir_path.join(INTERNAL_SOURCE_DIR);

    if !destination_root.exists() {
      fs::create_dir_all(&destination_root)?;
    }

    let mut unread = project_config.sources.smart_contracts.iter()
      .map(|pattern| {
        // `pattern` could use `/` or `\`, decomposing and collecting it normalizes
        // it into the correct format depending on plattform.
        let pattern = PathBuf::from(pattern).components().collect::<PathBuf>();
        let full_pattern = self.config.project_path.join(&pattern);
        info!("Searching for source files with pattern: {:?}", &full_pattern.display().to_string());
        full_pattern
      })
      .flat_map(|path| glob(&path.display().to_string()).unwrap().filter_map(Result::ok))
      .map(|path| {
        // Source file paths for compilation have to be absolute and canonicalized
        // (e.g. all `..` and `./` etc. removed) otherwise solcjs won't resolve and recognize
        // the source path properly. For more info see: https://github.com/ethereum/solc-js/issues/377
        //
        // In addition, on Windows platforms, the canonicalized path may include a verbatim (`\\?\`).
        // This breaks compilers (Solc, SolcJS), so we have to strip it out.
        lib_utils::adjust_canonicalization(&path.canonicalize().unwrap())
      })
      .collect::<Vec<PathBuf>>();

    let mut seen = unread.iter().cloned().collect::<HashSet<_>>();
    let mut normalized_imports = HashSet::new();

    while let Some(path) = unread.pop() {
      if let Ok(mut contents) = fs::read_to_string(&path) {
        let destination_path = utils::get_destination_path(&path, &self.config.project_path, &destination_root);
        fs::create_dir_all(destination_path.parent().unwrap())?;

        let imports = utils::extract_imports(&mut contents);
        let mut normalized_file = fs::File::create(&destination_path)?;

        info!("Normalizing imports for: {:?}", &path);
        for import in imports {
          let resolved_import = utils::resolve_import(&import, &path.parent().unwrap(), &self.config.project_path, &destination_root)?;

          contents = contents.replace(&import, &resolved_import.2.to_str().unwrap());
          if !seen.contains(&resolved_import.1) {
            unread.push(resolved_import.1.clone());
            seen.insert(resolved_import.1);
          }
        }
        normalized_file.write_all(contents.as_bytes())?;
        if !normalized_imports.contains(&destination_path) {
          normalized_imports.insert(destination_path);
        }
      }
    }
    Ok(normalized_imports.iter().cloned().collect::<Vec<PathBuf>>())
  }

  pub fn compile(&self, config: CompilerConfig) -> Result<Child, error::CompilerError> {
    let project_config = self.config.read()?;
    // `project_config.sources.artifacts` could use `/` or `\`, decomposing and collecting it normalizes
    // it into the correct format depending on plattform.
    let artifacts_path = PathBuf::from(&project_config.sources.artifacts).components().collect::<PathBuf>();
    let artifacts_dir = self.config.project_path.join(artifacts_path);

    let compiler = config.compiler.unwrap_or_else(|| {
      match &project_config.compiler {
        Some(config) => config.cmd.clone().unwrap_or_else(|| SupportedCompilers::Solc.executable()),
        None => SupportedCompilers::Solc.executable(),
      }
    });

    let mut compiler_options = match &config.compiler_options {
      Some(options) => {
        match compiler.parse() {
          Ok(SupportedCompilers::Solc) => lib_utils::merge_cli_options(
            support::default_options_from(SupportedCompilers::Solc),
            options.to_vec()
          ),
          Ok(SupportedCompilers::SolcJs) => lib_utils::merge_cli_options(
            support::default_options_from(SupportedCompilers::SolcJs),
            options.to_vec()
          ),
          Err(_err) => options.to_vec(),
        }
      }
      None => {
        match project_config.compiler {
          Some(config) => config.options.unwrap_or_else(|| try_default_options_from(&compiler)),
          None => try_default_options_from(&compiler)
        }
      }
    };

    if compiler_options.is_empty() {
      if let Err(err) = compiler.parse::<SupportedCompilers>() {
        Err(err)?
      }
    }

    compiler_options.push(artifacts_dir.to_string_lossy().to_string());

    let input_files = match compiler.parse() {
      Ok(SupportedCompilers::Solc) => {
        self.normalize_imports()?
          .iter()
          .map(|path| path.to_string_lossy().to_string())
          .collect::<Vec<String>>()
      },
      Ok(SupportedCompilers::SolcJs) => {
        self.normalize_imports()?
          .iter()
          .map(|path| path.to_string_lossy().to_string())
          .collect::<Vec<String>>()
      },
      Err(_err) => {
        let mut files = vec![];
        for pattern in &project_config.sources.smart_contracts {
          let pattern = PathBuf::from(&pattern);
          let full_pattern = if pattern.starts_with(&self.config.project_path) {
            pattern
          } else {
            self.config.project_path.join(&pattern)
          };
          files.extend(
            glob(&full_pattern.to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<String>>()
          );
        }
        files
      }
    };

    compiler_options.extend(input_files);
    compiler_options.insert(0, compiler.to_string());

    let (shell, shell_opt) = if cfg!(target_os = "windows") {
      ("cmd", "/C")
    } else {
      ("sh", "-c")
    };

    info!("Compiling project using command: {} {} {}", &shell, &shell_opt, compiler_options.join(" "));

    Command::new(shell)
      .arg(shell_opt)
      .arg(&compiler_options.join(" "))
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|err| {
        match err.kind() {
          std::io::ErrorKind::NotFound => error::CompilerError::ExecutableNotFound(err, shell.to_owned()),
          _ => error::CompilerError::Io(err)
        }
      })
  }
}

fn try_default_options_from(compiler: &str) -> Vec<String> {
  match compiler.parse() {
    Ok(SupportedCompilers::Solc) => support::default_options_from(SupportedCompilers::Solc),
    Ok(SupportedCompilers::SolcJs) => support::default_options_from(SupportedCompilers::SolcJs),
    Err(_err) => vec![],
  }
}

