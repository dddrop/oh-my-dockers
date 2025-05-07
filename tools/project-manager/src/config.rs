use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub global: GlobalSettings,
    #[serde(default)]
    pub defaults: DefaultSettings,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GlobalSettings {
    pub caddy_network: String,
    pub cert_dir: String,
    pub projects_dir: String,
    pub templates_dir: String,
    pub init_dir: String,
    pub caddy_projects_dir: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct DefaultSettings {
    pub postgres_version: Option<String>,
    pub redis_version: Option<String>,
    pub surrealdb_version: Option<String>,
    pub chroma_version: Option<String>,
    pub ollama_version: Option<String>,
    pub n8n_version: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,
    pub network: NetworkConfig,
    pub caddy: CaddyConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub domain: String,
    pub mode: String,
    #[serde(default)]
    pub port_offset: u16,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    #[serde(default)]
    pub enabled: bool,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub external: bool,
}

#[derive(Debug, Deserialize)]
pub struct CaddyConfig {
    #[serde(default)]
    pub auto_subdomains: bool,
    #[serde(default)]
    pub routes: Vec<CaddyRoute>,
}

#[derive(Debug, Deserialize)]
pub struct CaddyRoute {
    pub domain: Option<String>,
    pub subdomain: Option<String>,
    pub target: String,
}

#[allow(dead_code)]
pub fn load_global_config() -> Result<GlobalConfig> {
    let config_path = Path::new("config.toml");
    let content = fs::read_to_string(config_path).context("Failed to read config.toml")?;

    toml::from_str(&content).context("Failed to parse config.toml")
}

pub fn load_project_config(project: &str) -> Result<ProjectConfig> {
    let config_path = format!("projects/{}.toml", project);
    let content = fs::read_to_string(&config_path).context(format!(
        "Failed to read project configuration: {}",
        config_path
    ))?;

    toml::from_str(&content).context("Failed to parse project configuration")
}

pub fn load_project_env(project: &str) -> Result<HashMap<String, String>> {
    let env_path = format!("projects/{}/.env", project);
    let mut env_vars = HashMap::new();

    if Path::new(&env_path).exists() {
        let content = fs::read_to_string(&env_path).context("Failed to read .env file")?;

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
