//! Project configuration (omd.toml)
//!
//! This module handles loading and parsing project-level configuration files.

use std::{collections::HashMap, env, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Project configuration from omd.toml
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub network: NetworkConfig,
    #[serde(default)]
    pub caddy: CaddyConfig,
}

/// Default docker-compose file name
fn default_compose_file() -> String {
    "docker-compose.yml".to_string()
}

/// Project information
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub domain: String,
    /// Path to docker-compose file (relative to project directory)
    /// Defaults to "docker-compose.yml" if not specified
    #[serde(default = "default_compose_file")]
    pub compose_file: String,
}

/// Network configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub name: String,
}

/// Caddy configuration for the project
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CaddyConfig {
    /// Custom routes mapping: subdomain/path -> container:port
    #[serde(default)]
    pub routes: HashMap<String, String>,
}

/// Load project configuration from a specific path
pub fn load_project_config_from_path(path: &Path) -> Result<ProjectConfig> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read project configuration: {:?}", path))?;

    toml::from_str(&content).context("Failed to parse project configuration")
}

/// Load project configuration from current directory
pub fn load_project_config() -> Result<ProjectConfig> {
    let config_path = Path::new("omd.toml");

    if !config_path.exists() {
        anyhow::bail!("No omd.toml found in current directory. Run 'omd init' to create one.");
    }

    load_project_config_from_path(config_path)
}

/// Get the current directory name (for default project naming)
pub fn get_current_dir_name() -> Result<String> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let dir_name = current_dir
        .file_name()
        .context("Failed to get directory name")?
        .to_string_lossy()
        .to_string();

    Ok(dir_name)
}
