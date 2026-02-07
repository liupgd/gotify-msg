use anyhow::Result;
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub token: String,
    pub timeout_seconds: u64,
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout_seconds == 0 {
            return Err("超时时间必须大于 0".to_string());
        }

        if self.timeout_seconds > 300 {
            return Err("超时时间不能超过 300 秒".to_string());
        }

        if !self.server_url.trim().is_empty()
            && !self.server_url.starts_with("http://")
            && !self.server_url.starts_with("https://")
            && !self.server_url.starts_with("ws://")
            && !self.server_url.starts_with("wss://")
        {
            return Err("服务器地址必须以 http://, https://, ws:// 或 wss:// 开头".to_string());
        }

        Ok(())
    }
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

    // 验证加载的配置
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("配置文件验证失败: {}", e))?;

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    // 验证配置
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("配置验证失败: {}", e))?;

    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;

    let config_path = get_config_path();
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content)?;

    Ok(())
}
