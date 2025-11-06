mod caddy;

use std::{env, fs};

use anyhow::{Context, Result};
use colored::Colorize;

use crate::{
    config::{get_config_dir, load_global_config, load_project_config},
    docker_compose::ComposeInfo,
    hosts,
    network::{connect_caddy_to_network, ensure_network},
    proxy,
    registry::{PortRegistry, ProjectEntry},
};

/// List all registered projects
pub fn list() -> Result<()> {
    println!("{}", "Registered projects:".blue());
    println!();

    let registry = PortRegistry::load()?;
    let projects = registry.list_projects();

    if projects.is_empty() {
        println!("{}", "No projects registered".yellow());
        println!();
        println!("To register a project:");
        println!("  1. Navigate to your project directory");
        println!("  2. Run {}", "omd init".bright_white());
        println!("  3. Run {}", "omd up".bright_white());
        return Ok(());
    }

    for entry in projects {
        println!("  {} {}", "•".bright_white(), entry.name.bright_white());
        println!("    Path: {}", entry.path.display());
        println!("    Domain: {}", entry.domain);
        println!("    Network: {}", entry.network);
        if !entry.ports.is_empty() {
            println!("    Ports: {}", format_ports(&entry.ports));
        }
            println!();
    }

    Ok(())
}

/// Configure and register a project (run from project directory)
pub fn up() -> Result<()> {
    println!("{}", "Configuring project...".blue());

    // Load project configuration from current directory
    let mut config = load_project_config()?;

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Set project path
    config.project.path = Some(current_dir.to_string_lossy().to_string());

    println!(
        "{} Project: {}",
        "ℹ".blue(),
        config.project.name.bright_white()
    );
    println!("{} Domain: {}", "ℹ".blue(), config.project.domain);
    println!("{} Network: {}", "ℹ".blue(), config.network.name);

    // Check for docker-compose file
    let compose_path = current_dir.join(&config.project.compose_file);
    if !compose_path.exists() {
        anyhow::bail!(
            "docker-compose file not found: {}\n\
            Please ensure the file exists or update the 'compose_file' setting in omd.toml.",
            compose_path.display()
        );
    }

    // Parse docker-compose file
    println!("{} Parsing {}...", "ℹ".blue(), config.project.compose_file);
    let compose_info = ComposeInfo::parse(&compose_path)?;

    // Get all host ports
    let host_ports = compose_info.get_all_host_ports();

    if !host_ports.is_empty() {
        println!(
            "{} Found host ports: {}",
            "ℹ".blue(),
            format_ports(&host_ports)
        );
    } else {
        println!("{} No host port mappings found", "ℹ".blue());
    }

    // Get all container names
    let container_names = compose_info.get_all_container_names(&config.project.name);
    println!(
        "{} Container names: {}",
        "ℹ".blue(),
        container_names.join(", ")
    );

    // Check for port conflicts
    let mut registry = PortRegistry::load()?;
    let conflicts = registry.check_port_conflicts(&config.project.name, &host_ports);

    if !conflicts.is_empty() {
        println!();
        println!("{} Port conflicts detected:", "✗".red());
        for (port, project) in &conflicts {
            println!(
                "  Port {} is already used by project {}",
                port.to_string().red(),
                project.bright_white()
            );
        }
        println!();
        anyhow::bail!(
            "Cannot proceed due to port conflicts. Please update your docker-compose.yml to use different ports."
        );
    }

    println!("{} No port conflicts", "✓".green());

    // Ensure networks exist
    let global_config = load_global_config()?;
    
    // Create all globally defined networks
    for (network_name, _network_def) in &global_config.networks {
        ensure_network(network_name)?;
    }
    
    // Create project network
        ensure_network(&config.network.name)?;

    // Auto-start Caddy if not running
    crate::caddy_manager::auto_start_if_needed()?;

    // Generate Caddy configuration
    caddy::generate_caddy_config(&config, &compose_info)?;

    // Connect Caddy to project network
    connect_caddy_to_network(&config.network.name)?;

    // Register project in port registry
    let entry = ProjectEntry {
        name: config.project.name.clone(),
        path: current_dir.clone(),
        domain: config.project.domain.clone(),
        network: config.network.name.clone(),
        ports: host_ports,
        containers: container_names,
    };

    registry.register_project(entry)?;

    // Reload Caddy
    proxy::reload()?;

    // Update /etc/hosts with project domains
    println!();
    println!("{} Updating /etc/hosts...", "ℹ".blue());
    
    // Collect all domains (main domain + custom routes + auto-generated routes)
    let mut domains = vec![config.project.domain.clone()];
    
    // Add custom routes
    for (subdomain, _) in &config.caddy.routes {
        domains.push(format!("{}.{}", subdomain, config.project.domain));
    }
    
    // Add auto-generated routes (if no custom routes)
    if config.caddy.routes.is_empty() {
        for (service_name, service_info) in &compose_info.services {
            // Only add services with container ports (HTTP services)
            if !service_info.container_ports.is_empty() {
                domains.push(format!("{}.{}", service_name, config.project.domain));
            }
        }
    }
    
    if let Err(e) = hosts::add_project_domains(&config.project.name, &domains) {
        // Log error but don't fail the entire operation
        println!("{} Warning: Failed to update /etc/hosts: {}", "⚠".yellow(), e);
    }

    println!();
    println!(
        "{} Project {} is configured!",
        "✓".green(),
        config.project.name.bright_white()
    );
    println!();
    println!("Next steps:");
    println!(
        "  1. Run {} to start your services",
        "docker compose up -d".bright_white()
    );
    println!(
        "  2. Access your project at: https://{}",
        config.project.domain
    );
    if !config.caddy.routes.is_empty() {
        println!();
        println!("Custom routes:");
        for (subdomain, _) in &config.caddy.routes {
            println!("  - https://{}.{}", subdomain, config.project.domain);
        }
    }

    Ok(())
}

/// Remove project configuration (run from project directory)
pub fn down() -> Result<()> {
    println!("{}", "Removing project configuration...".blue());

    // Load project configuration
    let config = load_project_config()?;

        println!(
        "{} Project: {}",
        "ℹ".blue(),
        config.project.name.bright_white()
        );

    // Remove Caddy configuration
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let caddy_config = config_dir
        .join(&global_config.global.caddy_projects_dir)
        .join(format!("{}.caddy", config.project.name));

    if caddy_config.exists() {
        fs::remove_file(&caddy_config).context("Failed to remove Caddy configuration")?;
        println!("{} Removed Caddy configuration", "✓".green());
    }

    // Unregister from port registry
    let mut registry = PortRegistry::load()?;
    registry.unregister_project(&config.project.name)?;
    println!("{} Unregistered project", "✓".green());

    // Remove domains from /etc/hosts
    println!("{} Removing project domains from /etc/hosts...", "ℹ".blue());
    if let Err(e) = hosts::remove_project_domains(&config.project.name) {
        // Log error but don't fail the entire operation
        println!("{} Warning: Failed to update /etc/hosts: {}", "⚠".yellow(), e);
    }

    // Reload Caddy
    proxy::reload()?;

    println!();
    println!(
        "{} Project {} configuration removed",
        "✓".green(),
        config.project.name.bright_white()
    );
    println!();
    println!("Note: Docker containers are still running.");
    println!("Run {} to stop them.", "docker compose down".bright_white());

    Ok(())
}

/// Format a list of ports for display
fn format_ports(ports: &[u16]) -> String {
    if ports.is_empty() {
        return "none".to_string();
    }

    let mut formatted = ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    if formatted.len() > 60 {
        formatted.truncate(60);
        formatted.push_str("...");
    }

    formatted
}
