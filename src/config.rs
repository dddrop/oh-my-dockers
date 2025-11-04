use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result};
use dirs;
use serde::{Deserialize, Serialize};

/// Get the configuration directory path
/// Checks OH_MY_DOCKERS_DIR environment variable first,
/// then defaults to ~/.oh-my-dockers
pub fn get_config_dir() -> Result<PathBuf> {
    if let Ok(custom_dir) = env::var("OH_MY_DOCKERS_DIR") {
        return Ok(PathBuf::from(custom_dir));
    }

    let home_dir = dirs::home_dir()
        .context("Failed to get home directory")?;
    
    Ok(home_dir.join(".oh-my-dockers"))
}

/// Ensure the configuration directory and all subdirectories exist
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;

    // Create main directory
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    // Create subdirectories
    let subdirs = [
        "projects",
        "caddy",
        "caddy/certs",
        "caddy/projects",
        "templates",
        "init",
        "generated",
    ];

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
projects_dir = "projects"
templates_dir = "templates"
init_dir = "init"
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
# Default versions for services
postgres_version = "latest"
redis_version = "latest"
surrealdb_version = "latest"
chroma_version = "latest"
ollama_version = "latest"
n8n_version = "latest"

# Default timezone
timezone = "Asia/Tokyo"
"#;

    fs::write(config_path, default_config)
        .context("Failed to write default config file")?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub global: GlobalSettings,
    #[serde(default)]
    pub defaults: DefaultSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalSettings {
    pub caddy_network: String,
    pub projects_dir: String,
    pub templates_dir: String,
    pub init_dir: String,
    pub caddy_projects_dir: String,
    pub caddy_certs_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DefaultSettings {
    pub postgres_version: Option<String>,
    pub redis_version: Option<String>,
    pub surrealdb_version: Option<String>,
    pub chroma_version: Option<String>,
    pub ollama_version: Option<String>,
    pub n8n_version: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,
    pub network: NetworkConfig,
    pub caddy: CaddyConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub domain: String,
    pub mode: String,
    #[serde(default)]
    pub port_offset: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceConfig {
    #[serde(default)]
    pub enabled: bool,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub name: String,
    #[serde(default)]
    pub external: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CaddyConfig {
    #[serde(default)]
    pub auto_subdomains: bool,
    #[serde(default)]
    pub routes: Vec<CaddyRoute>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CaddyRoute {
    pub domain: Option<String>,
    pub subdomain: Option<String>,
    pub target: String,
}

/// Load global configuration
pub fn load_global_config() -> Result<GlobalConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    let content = fs::read_to_string(&config_path)
        .context(format!("Failed to read config.toml from {:?}", config_path))?;

    toml::from_str(&content).context("Failed to parse config.toml")
}

/// Load project configuration
pub fn load_project_config(project: &str) -> Result<ProjectConfig> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let projects_dir = config_dir.join(&global_config.global.projects_dir);
    let config_path = projects_dir.join(format!("{}.toml", project));
    
    let content = fs::read_to_string(&config_path).context(format!(
        "Failed to read project configuration: {:?}",
        config_path
    ))?;

    toml::from_str(&content).context("Failed to parse project configuration")
}

/// List all project configurations
pub fn list_projects() -> Result<Vec<String>> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let projects_dir = config_dir.join(&global_config.global.projects_dir);
    
    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    let entries = fs::read_dir(&projects_dir)
        .context("Failed to read projects directory")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(ext) = path.extension() {
            if ext == "toml" {
                if let Some(stem) = path.file_stem() {
                    projects.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(projects)
}

/// Load environment variables from project .env file
pub fn load_project_env(project: &str) -> Result<HashMap<String, String>> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let projects_dir = config_dir.join(&global_config.global.projects_dir);
    let env_path = projects_dir.join(project).join(".env");
    let mut env_vars = HashMap::new();

    if env_path.exists() {
        let content = fs::read_to_string(&env_path)
            .context("Failed to read .env file")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                env_vars.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }

    Ok(env_vars)
}
