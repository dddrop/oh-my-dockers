use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

/// Create a new Docker network
pub fn create(name: &str) -> Result<()> {
    // Check if network exists
    let output = Command::new("docker")
        .args(&["network", "inspect", name])
        .output()
        .context("Failed to inspect network")?;

    if output.status.success() {
        println!("{} Network {} already exists", "ℹ".blue(), name.bright_white());
    } else {
        println!("{} Creating network {}...", "ℹ".blue(), name.bright_white());
        let status = Command::new("docker")
            .args(&["network", "create", name])
            .status()
            .context("Failed to create network")?;

        if !status.success() {
            anyhow::bail!("Failed to create network {}", name);
        }

        println!("{} Network {} created", "✓".green(), name.bright_white());
    }

    Ok(())
}

/// List all Docker networks
pub fn list() -> Result<()> {
    println!("{}", "Docker Networks:".blue());
    println!();

    let output = Command::new("docker")
        .args(&["network", "ls", "--format", "{{.Name}}\t{{.Driver}}\t{{.Scope}}"])
        .output()
        .context("Failed to list networks")?;

    if !output.status.success() {
        anyhow::bail!("Failed to list networks");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        println!("{}", "No networks found".yellow());
        return Ok(());
    }

    // Print header
    println!("  {:<30} {:<15} {}", "NAME", "DRIVER", "SCOPE");
    println!("  {}", "-".repeat(60));

    // Print networks
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            println!(
                "  {:<30} {:<15} {}",
                parts[0].bright_white(),
                parts[1],
                parts[2]
            );
        }
    }

    Ok(())
}

/// Remove a Docker network
#[allow(dead_code)]
pub fn remove(name: &str) -> Result<()> {
    // Check if network exists
    let output = Command::new("docker")
        .args(&["network", "inspect", name])
        .output()
        .context("Failed to inspect network")?;

    if !output.status.success() {
        println!("{} Network {} does not exist", "⚠".yellow(), name.bright_white());
        return Ok(());
    }

    println!("{} Removing network {}...", "ℹ".blue(), name.bright_white());
    let status = Command::new("docker")
        .args(&["network", "rm", name])
        .status()
        .context("Failed to remove network")?;

    if !status.success() {
        anyhow::bail!("Failed to remove network {}", name);
    }

    println!("{} Network {} removed", "✓".green(), name.bright_white());
    Ok(())
}

/// Connect a container to a network
#[allow(dead_code)]
pub fn connect(network: &str, container: &str) -> Result<()> {
    // Check if network exists
    let output = Command::new("docker")
        .args(&["network", "inspect", network])
        .output()
        .context("Failed to inspect network")?;

    if !output.status.success() {
        anyhow::bail!("Network {} does not exist", network);
    }

    // Check if container exists
    let container_output = Command::new("docker")
        .args(&["ps", "-a", "--filter", &format!("name={}", container), "--format", "{{.Names}}"])
        .output()
        .context("Failed to check container")?;

    let container_exists = String::from_utf8_lossy(&container_output.stdout)
        .trim()
        .contains(container);

    if !container_exists {
        anyhow::bail!("Container {} does not exist", container);
    }

    println!(
        "{} Connecting container {} to network {}...",
        "ℹ".blue(),
        container.bright_white(),
        network.bright_white()
    );

    let status = Command::new("docker")
        .args(&["network", "connect", network, container])
        .status()
        .context("Failed to connect container to network")?;

    if !status.success() {
        anyhow::bail!("Failed to connect container {} to network {}", container, network);
    }

    println!(
        "{} Container {} connected to network {}",
        "✓".green(),
        container.bright_white(),
        network.bright_white()
    );

    Ok(())
}

/// Ensure a network exists (used internally by other modules)
pub fn ensure_network(network: &str) -> Result<()> {
    create(network)
}

/// Connect Caddy container to a network
pub fn connect_caddy_to_network(network: &str) -> Result<()> {
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
        println!(
            "{} Caddy is not running, skipping network connection",
            "⚠".yellow()
        );
        return Ok(());
    }

    println!("{} Connecting Caddy to network {}...", "ℹ".blue(), network);

    // Try to connect (ignore error if already connected)
    let _ = Command::new("docker")
        .args(&["network", "connect", network, "oh-my-dockers-caddy"])
        .output();

    Ok(())
}
