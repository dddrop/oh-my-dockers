use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

mod caddy;
mod compose;
mod config;
mod network;

use config::ProjectConfig;

#[derive(Parser)]
#[command(name = "project-manager")]
#[command(about = "Manage oh-my-dockers projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all configured projects
    List,
    /// Start a project
    Up {
        /// Project name
        project: String,
    },
    /// Stop a project
    Down {
        /// Project name
        project: String,
    },
    /// Reload Caddy configuration
    Reload,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_projects()?,
        Commands::Up { project } => start_project(&project)?,
        Commands::Down { project } => stop_project(&project)?,
        Commands::Reload => reload_caddy()?,
    }

    Ok(())
}

fn list_projects() -> Result<()> {
    println!("{}", "Available projects:".blue());
    println!();

    let projects_dir = Path::new("projects");
    if !projects_dir.exists() {
        println!("{}", "No projects directory found".yellow());
        return Ok(());
    }

    let entries = fs::read_dir(projects_dir).context("Failed to read projects directory")?;

    let mut found = false;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<ProjectConfig>(&content) {
                    let name = path.file_stem().unwrap().to_string_lossy();
                    println!("  {} {}", "•".bright_white(), name.bright_white());
                    println!("    Domain: {}", config.project.domain);
                    println!("    Mode: {}", config.project.mode);
                    println!();
                    found = true;
                }
            }
        }
    }

    if !found {
        println!("{}", "No projects found".yellow());
    }

    Ok(())
}

fn start_project(project: &str) -> Result<()> {
    println!("{} {}", "Starting project".blue(), project.bright_white());

    // Load project configuration
    let config = config::load_project_config(project)?;

    println!(
        "{} Project: {} ({} mode)",
        "ℹ".blue(),
        config.project.name,
        config.project.mode
    );
    println!("{} Domain: {}", "ℹ".blue(), config.project.domain);
    println!("{} Network: {}", "ℹ".blue(), config.network.name);

    // Ensure networks exist
    network::ensure_network("caddy-net")?;
    network::ensure_network(&config.network.name)?;

    // Load environment variables
    let env_vars = config::load_project_env(project)?;

    // Generate Caddy configuration
    caddy::generate_caddy_config(project, &config)?;

    if config.project.mode == "managed" {
        // Generate and start docker-compose
        let compose_file = compose::generate_compose_file(project, &config, &env_vars)?;

        println!("{} Starting services...", "ℹ".blue());
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
    network::connect_caddy_to_network(&config.network.name)?;

    // Reload Caddy
    reload_caddy()?;

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

fn stop_project(project: &str) -> Result<()> {
    println!("{} {}", "Stopping project".blue(), project.bright_white());

    // Load project configuration
    let config = config::load_project_config(project)?;

    if config.project.mode == "managed" {
        let compose_file = format!(".generated/docker-compose-{}.yml", project);

        if Path::new(&compose_file).exists() {
            println!("{} Stopping services...", "ℹ".blue());
            let status = Command::new("docker")
                .args(&["compose", "-f", &compose_file, "down"])
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
    let caddy_config = format!("caddy/projects/{}.caddy", project);
    if Path::new(&caddy_config).exists() {
        fs::remove_file(&caddy_config).context("Failed to remove Caddy configuration")?;
        println!("{} Removed Caddy configuration", "ℹ".blue());
    }

    println!("{} Project {} stopped", "✓".green(), project.bright_white());

    Ok(())
}

fn reload_caddy() -> Result<()> {
    // Check if Caddy is running
    let output = Command::new("docker")
        .args(&[
            "ps",
            "--filter",
            "name=oh-my-dockers-caddy",
            "--format",
            "{{.Names}}",
        ])
        .output()
        .context("Failed to check Caddy status")?;

    let caddy_running = String::from_utf8_lossy(&output.stdout)
        .trim()
        .contains("oh-my-dockers-caddy");

    if !caddy_running {
        println!("{} Caddy is not running, skipping reload", "⚠".yellow());
        return Ok(());
    }

    println!("{} Reloading Caddy configuration...", "ℹ".blue());
    let status = Command::new("docker")
        .args(&[
            "exec",
            "oh-my-dockers-caddy",
            "caddy",
            "reload",
            "--config",
            "/etc/caddy/Caddyfile",
        ])
        .status()
        .context("Failed to reload Caddy")?;

    if !status.success() {
        anyhow::bail!("Failed to reload Caddy configuration");
    }

    println!("{}", "✓ Caddy configuration reloaded".green());

    Ok(())
}
