use std::{fs, process::Command};

use anyhow::{Context, Result};
use colored::Colorize;

const HOSTS_PATH: &str = "/etc/hosts";
const TEMP_PATH: &str = "/tmp/hosts.oh-my-dockers.tmp";
const BACKUP_PATH: &str = "/etc/hosts.backup";

pub fn write_hosts_file(content: &str) -> Result<()> {
    println!();
    println!("{} /etc/hosts...", "Updating".blue());

    // Create backup first
    create_backup()?;

    // Write to temporary file
    fs::write(TEMP_PATH, content).context("Failed to write temporary file")?;

    // Validate temp file
    validate_hosts_file(TEMP_PATH)?;

    // Use sudo to copy temp file to /etc/hosts
    let status = Command::new("sudo")
        .args(&["cp", TEMP_PATH, HOSTS_PATH])
        .status()
        .context("Failed to execute sudo. Make sure sudo is available.")?;

    if !status.success() {
        // Cleanup temp file
        let _ = fs::remove_file(TEMP_PATH);
        anyhow::bail!("Failed to update /etc/hosts. Operation cancelled.");
    }

    // Cleanup temp file
    fs::remove_file(TEMP_PATH).context("Failed to remove temporary file")?;

    Ok(())
}

fn create_backup() -> Result<()> {
    let status = Command::new("sudo")
        .args(&["cp", HOSTS_PATH, BACKUP_PATH])
        .status()
        .context("Failed to create backup")?;

    if !status.success() {
        anyhow::bail!("Failed to create backup of /etc/hosts");
    }

    println!("{} Created backup: {}", "ℹ".blue(), BACKUP_PATH);

    Ok(())
}

fn validate_hosts_file(path: &str) -> Result<()> {
    let content =
        fs::read_to_string(path).context("Failed to read temporary file for validation")?;

    // Basic validation: check that file is not empty and has valid format
    if content.trim().is_empty() {
        anyhow::bail!("Generated hosts file is empty");
    }

    // Check for valid IP addresses and domains
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Basic format check: should have IP and at least one domain
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid hosts file format: {}", line);
        }

        // Check if first part looks like an IP
        if !parts[0].contains('.') && !parts[0].contains(':') {
            anyhow::bail!("Invalid IP address: {}", parts[0]);
        }
    }

    Ok(())
}
