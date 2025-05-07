use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

pub fn ensure_network(network: &str) -> Result<()> {
    // Check if network exists
    let output = Command::new("docker")
        .args(&["network", "inspect", network])
        .output()
        .context("Failed to inspect network")?;

    if output.status.success() {
        println!("{} Network {} already exists", "ℹ".blue(), network);
    } else {
        println!("{} Creating network {}...", "ℹ".blue(), network);
        let status = Command::new("docker")
            .args(&["network", "create", network])
            .status()
            .context("Failed to create network")?;

        if !status.success() {
            anyhow::bail!("Failed to create network {}", network);
        }

        println!("{} Network {} created", "✓".green(), network);
    }

    Ok(())
}

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
