use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct DiagonatorConfig {
    pub diagonator_path: String,
    pub diagonator_args: Vec<String>,
    pub socket_path: String,
}

impl Default for DiagonatorConfig {
    fn default() -> Self {
        let diagonator_path: String = match dirs::executable_dir() {
            Some(mut path) => {
                path.push("diagonator");
                path.to_string_lossy().to_string()
            }
            None => String::new(),
        };
        let mut socket_path = env::temp_dir();
        socket_path.push("diagonator-server.sock");
        let socket_path = socket_path.to_string_lossy().to_string();
        Self {
            diagonator_path,
            diagonator_args: Vec::new(),
            socket_path,
        }
    }
}

#[derive(Debug)]
pub enum LoadConfigError {
    ConfigDirNotFound,
    SerializationError(toml::ser::Error),
    DeserializationError(toml::de::Error),
    WriteError(PathBuf, std::io::Error),
    ReadError(PathBuf, std::io::Error),
    CreateDirError(PathBuf, std::io::Error),
}

impl Display for LoadConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigDirNotFound => {
                write!(f, "Unable to determine path to configuration directory")
            }
            Self::SerializationError(err) => {
                write!(f, "Received error '{}' when serializing configuration", err)
            }
            Self::DeserializationError(err) => {
                write!(
                    f,
                    "Received error '{}' when deserializing configuration",
                    err
                )
            }
            Self::WriteError(path, err) => {
                write!(
                    f,
                    "Received error '{}' when writing to file {}",
                    err,
                    path.display()
                )
            }
            Self::ReadError(path, err) => {
                write!(
                    f,
                    "Received error '{}' when reading from file {}",
                    err,
                    path.display()
                )
            }
            Self::CreateDirError(path, err) => {
                write!(
                    f,
                    "Received error '{}' when creating directory {}",
                    err,
                    path.display()
                )
            }
        }
    }
}
impl From<toml::ser::Error> for LoadConfigError {
    fn from(err: toml::ser::Error) -> Self {
        Self::SerializationError(err)
    }
}

impl From<toml::de::Error> for LoadConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::DeserializationError(err)
    }
}

fn make_default_config(config_file_path: &PathBuf) -> Result<(), LoadConfigError> {
    eprintln!(
        "Creating default configuration file at {}",
        config_file_path.display()
    );
    let contents = toml::to_string_pretty(&DiagonatorConfig::default())?;
    fs::write(config_file_path, contents)
        .map_err(|err| LoadConfigError::WriteError(config_file_path.clone(), err))
}

pub fn load_config() -> Result<DiagonatorConfig, LoadConfigError> {
    let mut config_file_path = dirs::config_dir().ok_or(LoadConfigError::ConfigDirNotFound)?;
    config_file_path.push("diagonator-server");
    fs::create_dir_all(&config_file_path)
        .map_err(|err| LoadConfigError::CreateDirError(config_file_path.clone(), err))?;
    config_file_path.push("config.toml");
    if !config_file_path.exists() {
        make_default_config(&config_file_path)?;
    }
    eprintln!("Loading configuration from {}", config_file_path.display());
    let contents = fs::read_to_string(&config_file_path)
        .map_err(|err| LoadConfigError::ReadError(config_file_path, err))?;

    let config = toml::from_str(&contents)?;
    Ok(config)
}
