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
            // Get container internal port for the service
            // Note: For HTTP services, this returns the container internal port which is NOT affected by port_offset.
            // Port offset only affects host port mappings in docker-compose (e.g., "${POSTGRES_PORT:-5432}:5432"),
            // but Caddy connects to containers within the Docker network using internal ports, which remain constant.
            let port = get_service_port(service_name, config.project.port_offset)?;

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
/// This function calculates the port used by Caddy to connect to services.
/// For HTTP services (n8n, chroma, surrealdb, ollama), this returns the container internal port,
/// which is NOT affected by port_offset because these services don't have port mappings.
/// 
/// Port offset only affects host port mappings for database services (postgres, redis)
/// in docker-compose (e.g., "${POSTGRES_PORT:-5432}:5432"), but Caddy connects to containers
/// within the Docker network using internal ports, which remain constant.
/// 
/// This function accepts port_offset for consistency with compose.rs, but for HTTP services
/// that Caddy proxies, the offset is not applied since they use fixed container internal ports.
fn get_service_port(service: &str, _port_offset: u16) -> Result<String> {
    // Base container internal ports for services
    // These are the ports exposed by containers within the Docker network
    let base_port = match service {
        "postgres" => 5432u32,
        "redis" => 6379u32,
        "surrealdb" => 8000u32,
        "chroma" => 8000u32,
        "ollama" => 11434u32,
        "n8n" => 5678u32,
        _ => 8080u32,
    };

    // For HTTP services that Caddy proxies, we use container internal ports which are not offset.
    // Port offset only affects host port mappings, not container internal ports.
    // This is consistent with how Docker networking works: containers communicate via internal ports.
    Ok(base_port.to_string())
}

