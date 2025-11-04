use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{get_config_dir, load_global_config, ProjectConfig};

pub fn generate_caddy_config(project: &str, config: &ProjectConfig) -> Result<()> {
    println!("{} Generating Caddy configuration...", "ℹ".blue());

    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let output_dir = config_dir.join(&global_config.global.caddy_projects_dir);
    fs::create_dir_all(&output_dir)
        .context("Failed to create caddy projects directory")?;

    let output_file = output_dir.join(format!("{}.caddy", project));
    let mut caddy_config = format!(
        "# Auto-generated Caddy configuration for {}\n# Domain: {}\n\n",
        project, config.project.domain
    );

    if config.project.mode == "managed" && config.caddy.auto_subdomains {
        // Generate subdomains for enabled services (HTTP only)
        // Note: Caddy connects to container internal ports, which are not affected by port_offset
        // Port offset only affects host port mappings, not container internal ports
        for (service_name, service_config) in &config.services {
            if !service_config.enabled {
                continue;
            }

            // Only generate Caddy config for HTTP services
            if !is_http_service(service_name) {
                continue;
            }

            let subdomain = service_name;
            let target = format!("{}-{}", config.project.name, service_name);
            // Get container internal port (not affected by port_offset)
            let port = get_service_port(service_name);

            let cert_name = config.project.domain.replace('.', "_");
            caddy_config.push_str(&format!(
                "{}.{} {{\n    tls /certs/{}.crt /certs/{}.key\n    reverse_proxy {}:{}\n}}\n\n",
                subdomain,
                config.project.domain,
                cert_name,
                cert_name,
                target,
                port
            ));
        }
    }

    if config.project.mode == "proxy-only" || !config.caddy.routes.is_empty() {
        // Add custom routes
        let cert_name = config.project.domain.replace('.', "_");
        for route in &config.caddy.routes {
            let full_domain = if let Some(subdomain) = &route.subdomain {
                format!("{}.{}", subdomain, config.project.domain)
            } else if let Some(domain) = &route.domain {
                domain.clone()
            } else {
                continue;
            };

            caddy_config.push_str(&format!(
                "{} {{\n    tls /certs/{}.crt /certs/{}.key\n    reverse_proxy {}\n}}\n\n",
                full_domain, cert_name, cert_name, route.target
            ));
        }
    }

    fs::write(&output_file, caddy_config)
        .context("Failed to write Caddy configuration")?;

    println!("{} Generated {:?}", "✓".green(), output_file);

    Ok(())
}

fn is_http_service(service: &str) -> bool {
    match service {
        "n8n" | "chroma" | "surrealdb" | "ollama" => true,
        "postgres" | "redis" => false,
        _ => true, // Assume unknown services are HTTP
    }
}

/// Get the container internal port for a service
/// 
/// Note: This returns the container internal port, which is NOT affected by port_offset.
/// Port offset only affects host port mappings (e.g., "${POSTGRES_PORT:-5432}:5432"),
/// but Caddy connects to containers within the Docker network using internal ports.
/// 
/// For HTTP services used by Caddy (n8n, chroma, surrealdb, ollama), these ports are
/// always the same regardless of port_offset configuration.
fn get_service_port(service: &str) -> &'static str {
    match service {
        "postgres" => "5432",
        "redis" => "6379",
        "surrealdb" => "8000",
        "chroma" => "8000",
        "ollama" => "11434",
        "n8n" => "5678",
        _ => "8080",
    }
}

