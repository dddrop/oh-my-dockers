use std::{collections::HashMap, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    project: ProjectInfo,
    #[serde(default)]
    services: HashMap<String, ServiceConfig>,
    caddy: CaddyConfig,
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    domain: String,
    mode: String,
}

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct CaddyConfig {
    #[serde(default)]
    auto_subdomains: bool,
    #[serde(default)]
    routes: Vec<CaddyRoute>,
}

#[derive(Debug, Deserialize)]
struct CaddyRoute {
    domain: Option<String>,
    subdomain: Option<String>,
}

pub fn generate_entry(project: &str) -> Result<String> {
    // Load project configuration
    let config_path = format!("projects/{}.toml", project);
    let content = fs::read_to_string(&config_path).context(format!(
        "Failed to read project configuration: {}",
        config_path
    ))?;

    let config: ProjectConfig =
        toml::from_str(&content).context("Failed to parse project configuration")?;

    // Collect all domains/subdomains
    let mut domains = vec![config.project.domain.clone()];

    // For managed mode with auto_subdomains
    if config.project.mode == "managed" && config.caddy.auto_subdomains {
        for (service_name, service_config) in &config.services {
            if service_config.enabled && is_http_service(service_name) {
                domains.push(format!("{}.{}", service_name, config.project.domain));
            }
        }
    }

    // For proxy-only mode or custom routes
    if config.project.mode == "proxy-only" || !config.caddy.routes.is_empty() {
        for route in &config.caddy.routes {
            if let Some(subdomain) = &route.subdomain {
                domains.push(format!("{}.{}", subdomain, config.project.domain));
            } else if let Some(domain) = &route.domain {
                if domain != &config.project.domain {
                    domains.push(domain.clone());
                }
            }
        }
    }

    // Generate hosts entry with all domains on one line
    let domains_line = domains.join(" ");
    let entry = format!(
        "# oh-my-dockers: {}\n127.0.0.1 {}\n# oh-my-dockers: end",
        project, domains_line
    );

    Ok(entry)
}

fn is_http_service(service: &str) -> bool {
    matches!(service, "n8n" | "chroma" | "surrealdb" | "ollama")
}
