//! Project initialization
//!
//! This module handles initializing a new project with omd.toml configuration
//! and optionally generating a docker-compose.yml file.

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::{Context, Result};
use colored::Colorize;

use super::compose_generator::{
    generate_compose_file, prompt_service_selection, resolve_service_ports,
};
use super::config::get_current_dir_name;
use super::registry::PortRegistry;

/// Initialize a new omd.toml configuration in the current directory
pub fn init() -> Result<()> {
    let config_path = Path::new("omd.toml");

    if config_path.exists() {
        println!(
            "{} {} already exists in current directory",
            "⚠".yellow(),
            "omd.toml".bright_white()
        );
        print!("Overwrite? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted".yellow());
            return Ok(());
        }
    }

    // Get default values
    let default_name = get_current_dir_name().unwrap_or_else(|_| "my-project".to_string());

    // Interactive prompts
    println!("{}", "Creating omd.toml configuration...".blue());
    println!();

    let project_name = prompt_with_default("Project name", &default_name)?;
    let domain = prompt_with_default("Domain", &format!("{}.local", project_name))?;
    let network = prompt_with_default("Network name", &format!("{}-net", project_name))?;
    let compose_file = prompt_with_default("Docker Compose file", "docker-compose.yml")?;

    // Check if compose file exists
    let compose_path = Path::new(&compose_file);
    let mut compose_created = false;

    if !compose_path.exists() {
        println!();
        println!(
            "{} {} does not exist",
            "ℹ".blue(),
            compose_file.bright_white()
        );
        print!("Create docker-compose.yml with common services? [Y/n]: ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        let should_create = confirm.trim().is_empty()
            || confirm.trim().eq_ignore_ascii_case("y")
            || confirm.trim().eq_ignore_ascii_case("yes");

        if should_create {
            // Show service selection
            let selections = prompt_service_selection()?;

            if !selections.is_empty() {
                // Load registry to check port conflicts
                let registry = PortRegistry::load().unwrap_or_default();

                // Resolve ports for selected services
                let selected_services = resolve_service_ports(&selections, &registry);

                // Generate docker-compose.yml
                generate_compose_file(compose_path, &project_name, &network, &selected_services)?;

                let service_names: Vec<&str> = selected_services
                    .iter()
                    .map(|s| s.template.display_name)
                    .collect();

                println!();
                println!(
                    "{} Created {} with: {}",
                    "✓".green(),
                    compose_file.bright_white(),
                    service_names.join(", ")
                );
                compose_created = true;
            } else {
                println!(
                    "{} No services selected, skipping docker-compose.yml creation",
                    "ℹ".blue()
                );
            }
        }
    }

    // Ask about Caddy routes configuration
    println!();
    print!("Do you want to configure Caddy routes now? [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let configure_routes = input.trim().eq_ignore_ascii_case("y");

    // Generate config content
    let mut config_content = format!(
        r#"# oh-my-dockers Project Configuration
# See https://github.com/your-repo/oh-my-dockers for more information

[project]
# Project name (used for container naming)
name = "{}"

# Domain for this project
domain = "{}"
"#,
        project_name, domain
    );

    // Only add compose_file if it's not the default
    if compose_file != "docker-compose.yml" {
        config_content.push_str(&format!(
            r#"
# Path to docker-compose file (relative to project directory)
compose_file = "{}"
"#,
            compose_file
        ));
    }

    config_content.push_str(&format!(
        r#"
[network]
# Docker network name for this project
name = "{}"
"#,
        network
    ));

    if configure_routes {
        config_content.push_str(
            r#"
[caddy]
# Custom Caddy routes
# Format: subdomain = "container_name:port"
# Example:
#   api = "backend:3000"
#   app = "frontend:80"
#   admin = "admin:8080"
routes = {}
"#,
        );
    }

    // Write config file
    fs::write(config_path, config_content).context("Failed to write omd.toml")?;

    println!();
    println!("{} Created {}", "✓".green(), "omd.toml".bright_white());
    println!();
    println!("Next steps:");

    if !compose_created && !compose_path.exists() {
        println!("  1. Create your docker-compose.yml");
        println!(
            "  2. Run {} to configure and start your services",
            "omd project up".bright_white()
        );
    } else {
        println!(
            "  1. Run {} to configure and start your services",
            "omd project up".bright_white()
        );
    }

    Ok(())
}

/// Prompt user for input with a default value
fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, default.bright_black());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}
