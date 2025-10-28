use std::{fs, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::ProjectConfig;

pub fn generate_caddy_config(project: &str, config: &ProjectConfig) -> Result<()> {
    println!("{} Generating Caddy configuration...", "ℹ".blue());

    let output_dir = Path::new("caddy/projects");
    fs::create_dir_all(output_dir).context("Failed to create caddy/projects directory")?;

    let output_file = format!("caddy/projects/{}.caddy", project);
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
            let port = get_service_port(service_name);

            caddy_config.push_str(&format!(
                "{}.{} {{\n    tls /certs/{}.crt /certs/{}.key\n    reverse_proxy {}:{}\n}}\n\n",
                subdomain,
                config.project.domain,
                config.project.domain,
                config.project.domain,
                target,
                port
            ));
        }
    }

    if config.project.mode == "proxy-only" || !config.caddy.routes.is_empty() {
        // Add custom routes
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
                full_domain, config.project.domain, config.project.domain, route.target
            ));
        }
    }

    fs::write(&output_file, caddy_config).context("Failed to write Caddy configuration")?;

    println!("{} Generated {}", "✓".green(), output_file);

    Ok(())
}

fn is_http_service(service: &str) -> bool {
    match service {
        "n8n" | "chroma" | "surrealdb" | "ollama" => true,
        "postgres" | "redis" => false,
        _ => true, // Assume unknown services are HTTP
    }
}

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
