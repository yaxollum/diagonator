use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct DiagonatorConfig {
    diagonator_path: String,
    diagonator_args: Vec<String>,
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
        Self {
            diagonator_path,
            diagonator_args: Vec::new(),
        }
    }
}

enum ServerError {
    NoConfigDir,
}

fn make_default_config(config_file_path: &PathBuf) {
    eprintln!(
        "Creating default config file at {}",
        config_file_path.display()
    );
    toml::to_string_pretty(&DiagonatorConfig::default());
}

fn load_config() -> Result<DiagonatorConfig, ServerError> {
    let mut config_file_path = dirs::config_dir().ok_or(ServerError::NoConfigDir)?;
    config_file_path.push("diagonator-server");
    config_file_path.push("config.toml");
    if !config_file_path.exists() {
        make_default_config(&config_file_path);
    }
    let config_file = File::open(config_file_path);
    Ok(DiagonatorConfig::default())
}

fn main() {}
