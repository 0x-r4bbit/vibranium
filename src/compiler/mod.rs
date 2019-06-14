pub mod error;
pub mod support;

use std::process::{Child, Command, Stdio};
use crate::config;
use crate::utils;
use support::SupportedCompilers;
use glob::glob;

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

  pub fn compile(&self, config: CompilerConfig) -> Result<Child, error::CompilerError> {
    let project_config = self.config.read().map_err(error::CompilerError::InvalidConfig)?;
    let artifacts_dir = self.config.project_path.join(&project_config.sources.artifacts);

    let compiler = config.compiler.unwrap_or_else(|| {
      match &project_config.compiler {
        Some(config) => config.cmd.clone().unwrap_or_else(|| SupportedCompilers::Solc.executable()),
        None => SupportedCompilers::Solc.executable(),
      }
    });

    let mut compiler_options = match &config.compiler_options {
      Some(options) => {
        match compiler.parse() {
          Ok(SupportedCompilers::Solc) => utils::merge_cli_options(
            support::default_options_from(SupportedCompilers::Solc),
            options.to_vec()
          ),
          Ok(SupportedCompilers::SolcJs) => utils::merge_cli_options(
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

    for pattern in &project_config.sources.smart_contracts {
      let mut full_pattern = self.config.project_path.clone();
      full_pattern.push(&pattern);
      for entry in glob(&full_pattern.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        compiler_options.push(entry.to_string_lossy().to_string());
      }
    }

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
