use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::{
    config::{ProjectConfig, get_config_dir, load_global_config},
    docker_compose::ComposeInfo,
};

pub fn generate_caddy_config(config: &ProjectConfig, compose_info: &ComposeInfo) -> Result<()> {
    println!("{} Generating Caddy configuration...", "ℹ".blue());

    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let output_dir = config_dir.join(&global_config.global.caddy_projects_dir);
    fs::create_dir_all(&output_dir).context("Failed to create caddy projects directory")?;

    let output_file = output_dir.join(format!("{}.caddy", config.project.name));
    let mut caddy_config = format!(
        "# Auto-generated Caddy configuration for {}\n# Domain: {}\n\n",
        config.project.name, config.project.domain
    );

    // Generate routes based on user configuration
    if !config.caddy.routes.is_empty() {
        println!("{} Adding custom routes...", "ℹ".blue());

        for (subdomain, target) in &config.caddy.routes {
            let full_domain = format!("{}.{}", subdomain, config.project.domain);

            caddy_config.push_str(&format!(
                "{} {{\n    reverse_proxy {}\n}}\n\n",
                full_domain, target
            ));

            println!("  {} -> {}", full_domain.bright_white(), target);
        }
    } else {
        // Auto-generate routes from docker-compose services
        println!(
            "{} Auto-generating routes from docker-compose.yml...",
            "ℹ".blue()
        );

        for (service_name, service_info) in &compose_info.services {
            // Skip services without container ports (like databases without HTTP interface)
            if service_info.container_ports.is_empty() {
                continue;
            }

            // Use the first container port as default
            let port = service_info.container_ports[0];

            // Determine container name
            let container_name = service_info
                .container_name
                .clone()
                .unwrap_or_else(|| format!("{}-{}-1", config.project.name, service_name));

            let subdomain = service_name;
            let full_domain = format!("{}.{}", subdomain, config.project.domain);
            let target = format!("{}:{}", container_name, port);

            caddy_config.push_str(&format!(
                "{} {{\n    reverse_proxy {}\n}}\n\n",
                full_domain, target
            ));

            println!("  {} -> {}", full_domain.bright_white(), target);
        }
    }

    fs::write(&output_file, caddy_config).context("Failed to write Caddy configuration")?;

    println!("{} Generated {:?}", "✓".green(), output_file);

    Ok(())
}
