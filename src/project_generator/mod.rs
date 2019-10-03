extern crate log;
extern crate toml;

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::config;
use crate::deployment::tracker::TRACKING_FILE;

pub mod error;

pub const VIBRANIUM_PROJECT_DIRECTORY: &str = ".vibranium";
pub const DEFAULT_DATADIR_NAME: &str = "datadir";
pub const DEFAULT_DEV_PASSWORDS_DIR: &str = "passwords";
pub const DEFAULT_ENVIRONMENT: &str = "development";

pub struct ProjectGenerator<'a> {
    config: &'a config::Config,
}

pub struct ResetOptions {
    pub restore_config: bool,
    pub tracking_data_only: bool,
}

impl<'a> ProjectGenerator<'a> {
    pub fn new(config: &config::Config) -> ProjectGenerator {
        ProjectGenerator { config }
    }

    pub fn generate_project(
        &self,
        project_path: &PathBuf,
    ) -> Result<(), error::ProjectGenerationError> {
        if !project_path.exists() {
            return Err(error::ProjectGenerationError::ProjectPathNotFound);
        }

        let mut directories_to_create: Vec<PathBuf> = vec![
            project_path.join(VIBRANIUM_PROJECT_DIRECTORY),
            project_path
                .join(VIBRANIUM_PROJECT_DIRECTORY)
                .join(DEFAULT_DATADIR_NAME)
                .join(DEFAULT_ENVIRONMENT),
            project_path
                .join(VIBRANIUM_PROJECT_DIRECTORY)
                .join(DEFAULT_DEV_PASSWORDS_DIR),
            project_path.join(config::DEFAULT_CONTRACTS_DIRECTORY),
        ];

        if !self.config.exists() {
            directories_to_create.push(project_path.join(config::DEFAULT_ARTIFACTS_DIRECTORY));
            self.create_default_config_file()?;
        } else {
            let existing_config = self.config.read()?;
            directories_to_create.push(project_path.join(existing_config.sources.artifacts));
        }

        for path in directories_to_create {
            if !path.exists() {
                info!("Creating: {}", path.to_str().unwrap());
                fs::create_dir_all(path)?;
            }
        }
        Ok(())
    }

    pub fn reset_project(
        &self,
        project_path: &PathBuf,
        options: ResetOptions,
    ) -> Result<(), error::ProjectGenerationError> {
        self.check_vibranium_dir_exists()?;
        let vibranium_project_directory = self.config.vibranium_dir_path.clone();
        let default_artifacts_directory = project_path.join(config::DEFAULT_ARTIFACTS_DIRECTORY);

        if options.tracking_data_only {
            let tracking_file = vibranium_project_directory.join(TRACKING_FILE);
            info!("Removing: {}", &tracking_file.to_str().unwrap());
            fs::remove_file(tracking_file)?;
        } else {
            if options.restore_config {
                info!("Restoring project's config file");
                self.create_default_config_file()?;
            }

            if self.config.exists() {
                let existing_config = self.config.read()?;
                if existing_config.sources.artifacts != config::DEFAULT_ARTIFACTS_DIRECTORY {
                    let artifacts_dir = project_path.join(existing_config.sources.artifacts);
                    info!("Removing: {}", &artifacts_dir.to_str().unwrap());
                    let _ = fs::remove_dir_all(&artifacts_dir);
                }
            }

            info!(
                "Removing: {}",
                &vibranium_project_directory.to_str().unwrap()
            );
            let _ = fs::remove_dir_all(vibranium_project_directory);
            info!(
                "Removing: {}",
                &default_artifacts_directory.to_str().unwrap()
            );
            let _ = fs::remove_dir_all(&default_artifacts_directory);
        }

        Ok(())
    }

    pub fn check_vibranium_dir_exists(&self) -> Result<(), error::ProjectGenerationError> {
        if !self.config.vibranium_dir_path.exists() {
            return Err(error::ProjectGenerationError::VibraniumDirectoryNotFound);
        }
        Ok(())
    }

    fn create_default_config_file(&self) -> Result<(), error::ProjectGenerationError> {
        let config = config::ProjectConfig::default();
        info!("Creating: {}", &self.config.config_file.to_str().unwrap());
        let config_toml = toml::to_string(&config)?;
        let mut config_file = fs::File::create(&self.config.config_file)?;
        config_file.write_all(config_toml.as_bytes())?;
        Ok(())
    }
}
