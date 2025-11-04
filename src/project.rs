mod compose;
mod caddy;

use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{load_project_config, load_project_env, list_projects};
use crate::network::{ensure_network, connect_caddy_to_network};
use crate::proxy;

/// List all projects
pub fn list() -> Result<()> {
    println!("{}", "Available projects:".blue());
    println!();

    let projects = list_projects()?;

    if projects.is_empty() {
        println!("{}", "No projects found".yellow());
        return Ok(());
    }

    for project_name in &projects {
        if let Ok(config) = load_project_config(project_name) {
            println!("  {} {}", "•".bright_white(), project_name.bright_white());
            println!("    Domain: {}", config.project.domain);
            println!("    Mode: {}", config.project.mode);
            println!();
        }
    }

    Ok(())
}

/// Start a project
pub fn up(project: &str) -> Result<()> {
    println!("{} {}", "Starting project".blue(), project.bright_white());

    // Load project configuration
    let config = load_project_config(project)?;

    println!(
        "{} Project: {} ({} mode)",
        "ℹ".blue(),
        config.project.name,
        config.project.mode
    );
    println!("{} Domain: {}", "ℹ".blue(), config.project.domain);
    println!("{} Network: {}", "ℹ".blue(), config.network.name);

    // Ensure networks exist
    let global_config = crate::config::load_global_config()?;
    ensure_network(&global_config.global.caddy_network)?;
    ensure_network(&config.network.name)?;

    // Load environment variables
    let env_vars = load_project_env(project)?;

    // Generate Caddy configuration
    caddy::generate_caddy_config(project, &config)?;

    if config.project.mode == "managed" {
        // Generate and start docker-compose
        let compose_file = compose::generate_compose_file(project, &config, &env_vars)?;

        println!("{} Starting services...", "ℹ".blue());
        use std::process::Command;
        let status = Command::new("docker")
            .args(&["compose", "-f", &compose_file, "up", "-d"])
            .status()
            .context("Failed to start docker-compose")?;

        if !status.success() {
            anyhow::bail!("Failed to start services");
        }

        println!("{}", "✓ Services started".green());
    } else {
        println!(
            "{} Proxy-only mode: services managed externally",
            "ℹ".blue()
        );
    }

    // Connect Caddy to project network
    connect_caddy_to_network(&config.network.name)?;

    // Reload Caddy
    proxy::reload()?;

    println!();
    println!(
        "{} Project {} is ready!",
        "✓".green(),
        project.bright_white()
    );
    println!();
    println!("Access your services at:");
    println!("  https://{}", config.project.domain);

    Ok(())
}

/// Stop a project
pub fn down(project: &str) -> Result<()> {
    println!("{} {}", "Stopping project".blue(), project.bright_white());

    // Load project configuration
    let config = load_project_config(project)?;

    if config.project.mode == "managed" {
        let config_dir = crate::config::get_config_dir()?;
        let compose_file = config_dir
            .join("generated")
            .join(format!("docker-compose-{}.yml", project));

        if compose_file.exists() {
            println!("{} Stopping services...", "ℹ".blue());
            use std::process::Command;
            let status = Command::new("docker")
                .args(&["compose", "-f", compose_file.to_str().unwrap(), "down"])
                .status()
                .context("Failed to stop docker-compose")?;

            if !status.success() {
                anyhow::bail!("Failed to stop services");
            }

            println!("{}", "✓ Services stopped".green());
        } else {
            println!("{} No compose file found", "⚠".yellow());
        }
    } else {
        println!(
            "{} Proxy-only mode: services managed externally",
            "ℹ".blue()
        );
    }

    // Remove Caddy configuration
    let config_dir = crate::config::get_config_dir()?;
    let global_config = crate::config::load_global_config()?;
    let caddy_config = config_dir
        .join(&global_config.global.caddy_projects_dir)
        .join(format!("{}.caddy", project));

    if caddy_config.exists() {
        fs::remove_file(&caddy_config)
            .context("Failed to remove Caddy configuration")?;
        println!("{} Removed Caddy configuration", "ℹ".blue());
    }

    println!("{} Project {} stopped", "✓".green(), project.bright_white());

    Ok(())
}
