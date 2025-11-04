use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{get_config_dir, load_global_config};

/// Proxy rule storage
#[derive(Debug, Clone)]
struct ProxyRule {
    domain: String,
    target: String,
}

/// Add a reverse proxy rule
pub fn add(domain: &str, target: &str) -> Result<()> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let caddy_projects_dir = config_dir.join(&global_config.global.caddy_projects_dir);

    // Create a safe filename from domain
    let filename = domain.replace('.', "_").replace(':', "_");
    let config_file = caddy_projects_dir.join(format!("{}.caddy", filename));

    // Check if rule already exists
    if config_file.exists() {
        println!(
            "{} Proxy rule for {} already exists",
            "⚠".yellow(),
            domain.bright_white()
        );
        return Ok(());
    }

    // Generate Caddy configuration
    let cert_name = domain.replace('.', "_");
    let caddy_config = format!(
        "# Auto-generated proxy rule\n# Domain: {}\n# Target: {}\n\n{} {{\n    tls /certs/{}.crt /certs/{}.key\n    reverse_proxy {}\n}}\n",
        domain, target, domain, cert_name, cert_name, target
    );

    fs::write(&config_file, caddy_config).context("Failed to write proxy configuration")?;

    println!(
        "{} Added proxy rule: {} -> {}",
        "✓".green(),
        domain.bright_white(),
        target.bright_white()
    );

    // Reload Caddy if running
    reload()?;

    Ok(())
}

/// Remove a reverse proxy rule
pub fn remove(domain: &str) -> Result<()> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let caddy_projects_dir = config_dir.join(&global_config.global.caddy_projects_dir);

    // Try to find the config file
    let filename = domain.replace('.', "_").replace(':', "_");
    let config_file = caddy_projects_dir.join(format!("{}.caddy", filename));

    if !config_file.exists() {
        println!(
            "{} Proxy rule for {} not found",
            "⚠".yellow(),
            domain.bright_white()
        );
        return Ok(());
    }

    fs::remove_file(&config_file).context("Failed to remove proxy configuration")?;

    println!(
        "{} Removed proxy rule for {}",
        "✓".green(),
        domain.bright_white()
    );

    // Reload Caddy if running
    reload()?;

    Ok(())
}

/// List all proxy rules
pub fn list() -> Result<()> {
    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let caddy_projects_dir = config_dir.join(&global_config.global.caddy_projects_dir);

    if !caddy_projects_dir.exists() {
        println!("{}", "No proxy rules found".yellow());
        return Ok(());
    }

    println!("{}", "Proxy Rules:".blue());
    println!();

    let entries =
        fs::read_dir(&caddy_projects_dir).context("Failed to read caddy projects directory")?;

    let mut rules: Vec<ProxyRule> = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "caddy" {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Parse domain and target from config file
                    if let Some(rule) = parse_proxy_rule(&content) {
                        rules.push(rule);
                    }
                }
            }
        }
    }

    if rules.is_empty() {
        println!("{}", "No proxy rules found".yellow());
        return Ok(());
    }

    println!("  {:<40} {}", "DOMAIN", "TARGET");
    println!("  {}", "-".repeat(60));

    for rule in rules {
        println!("  {:<40} {}", rule.domain.bright_white(), rule.target);
    }

    Ok(())
}

/// Reload Caddy configuration
pub fn reload() -> Result<()> {
    use std::process::Command;

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

/// Parse proxy rule from Caddy config content
fn parse_proxy_rule(content: &str) -> Option<ProxyRule> {
    let mut domain = None;
    let mut target = None;

    for line in content.lines() {
        let line = line.trim();

        // Find domain (first non-comment line that's not empty and not a block start)
        if domain.is_none() && !line.is_empty() && !line.starts_with('#') && !line.starts_with('{')
        {
            if !line.contains("reverse_proxy") {
                // Extract domain by splitting on whitespace and removing trailing '{'
                let domain_str = line.split_whitespace().next().unwrap_or(line);
                let domain_clean = domain_str.trim_end_matches('{').trim();
                if !domain_clean.is_empty() {
                    domain = Some(domain_clean.to_string());
                }
            }
        }

        // Find target (reverse_proxy line)
        if line.contains("reverse_proxy") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(target_part) = parts.last() {
                target = Some(target_part.to_string());
            }
        }
    }

    if let (Some(domain), Some(target)) = (domain, target) {
        Some(ProxyRule { domain, target })
    } else {
        None
    }
}
