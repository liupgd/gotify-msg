use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub token: String,
    pub timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            token: String::new(),
            timeout_seconds: 5,
        }
    }
}

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gotify-client")
}

fn get_config_path() -> PathBuf {
    get_config_dir().join("config.yaml")
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: AppConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;
    
    let config_path = get_config_path();
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content)?;
    
    Ok(())
}

