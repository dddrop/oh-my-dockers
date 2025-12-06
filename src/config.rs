//! Global configuration management
//!
//! This module handles the global configuration for oh-my-dockers,
//! stored in ~/.oh-my-dockers/config.toml.

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Get the configuration directory path
/// Checks OH_MY_DOCKERS_DIR environment variable first,
/// then defaults to ~/.oh-my-dockers
pub fn get_config_dir() -> Result<PathBuf> {
    if let Ok(custom_dir) = env::var("OH_MY_DOCKERS_DIR") {
        return Ok(PathBuf::from(custom_dir));
    }

    let home_dir = dirs::home_dir().context("Failed to get home directory")?;

    Ok(home_dir.join(".oh-my-dockers"))
}

/// Ensure the configuration directory and all subdirectories exist
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;

    // Create main directory
    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    // Create subdirectories
    let subdirs = ["caddy", "caddy/certs", "caddy/projects"];

    for subdir in &subdirs {
        let dir_path = config_dir.join(subdir);
        fs::create_dir_all(&dir_path)
            .context(format!("Failed to create subdirectory: {}", subdir))?;
    }

    // Create default config.toml if it doesn't exist
    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        create_default_config(&config_file)?;
    }

    Ok(config_dir)
}

fn create_default_config(config_path: &Path) -> Result<()> {
    let default_config = r#"# Global Configuration for oh-my-dockers

[global]
# Caddy network name
caddy_network = "caddy-net"

# Directories (relative to config directory)
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

# Enable HTTPS for projects (default: false)
# When true, uses 'tls internal' for automatic local certificates
# Set to true to enable HTTPS with self-signed certificates for local domains
enable_https = true

[defaults]
# Default timezone
timezone = "Asia/Tokyo"

# Network definitions
# Networks are automatically created when running 'omd project up'
[networks]
# Caddy reverse proxy network
caddy-net = {}

# You can define additional networks with custom settings:
# my-network = { driver = "bridge", subnet = "172.20.0.0/16", gateway = "172.20.0.1" }
"#;

    fs::write(config_path, default_config).context("Failed to write default config file")?;

    Ok(())
}

/// Global configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub global: GlobalSettings,
    #[serde(default)]
    pub defaults: DefaultSettings,
    #[serde(default)]
    pub networks: HashMap<String, NetworkDefinition>,
}

/// Global settings section
#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalSettings {
    pub caddy_network: String,
    pub caddy_projects_dir: String,
    pub caddy_certs_dir: String,
    /// Enable HTTPS for projects (default: false)
    /// When true, uses 'tls internal' for automatic local certificates
    #[serde(default)]
    pub enable_https: bool,
}

/// Default settings section
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DefaultSettings {
    pub timezone: Option<String>,
}

/// Network definition for custom networks
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NetworkDefinition {
    pub driver: Option<String>,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
}

/// Load global configuration
pub fn load_global_config() -> Result<GlobalConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    let content = fs::read_to_string(&config_path)
        .context(format!("Failed to read config.toml from {:?}", config_path))?;

    let config: GlobalConfig = toml::from_str(&content).context("Failed to parse config.toml")?;

    Ok(config)
}
