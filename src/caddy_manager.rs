use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::get_config_dir;

/// Check if Caddy container is running
pub fn is_running() -> bool {
    let output = Command::new("docker")
        .args(&["ps", "--filter", "name=oh-my-dockers-caddy", "--format", "{{.Names}}"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("oh-my-dockers-caddy")
    } else {
        false
    }
}

/// Check if Caddy container exists (running or stopped)
fn container_exists() -> bool {
    let output = Command::new("docker")
        .args(&["ps", "-a", "--filter", "name=oh-my-dockers-caddy", "--format", "{{.Names}}"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("oh-my-dockers-caddy")
    } else {
        false
    }
}

/// Remove existing Caddy container
fn remove_container() -> Result<()> {
    println!("{} Removing existing container...", "ℹ".blue());
    let status = Command::new("docker")
        .args(&["rm", "-f", "oh-my-dockers-caddy"])
        .status()
        .context("Failed to remove container")?;

    if !status.success() {
        anyhow::bail!("Failed to remove existing container");
    }

    Ok(())
}

/// Start existing stopped container
fn start_existing_container() -> Result<()> {
    println!("{} Starting existing container...", "ℹ".blue());
    let status = Command::new("docker")
        .args(&["start", "oh-my-dockers-caddy"])
        .status()
        .context("Failed to start container")?;

    if !status.success() {
        anyhow::bail!("Failed to start existing container");
    }

    Ok(())
}

/// Ensure Caddyfile exists in config directory
fn ensure_caddyfile() -> Result<()> {
    let config_dir = get_config_dir()?;
    let caddyfile_path = config_dir.join("caddy/Caddyfile");

    if caddyfile_path.exists() {
        return Ok(());
    }

    println!("{} Creating Caddyfile...", "ℹ".blue());

    // Check if HTTPS is enabled in global config
    let global_config = crate::config::load_global_config().ok();
    let enable_https = global_config
        .as_ref()
        .map(|c| c.global.enable_https)
        .unwrap_or(false);

    let auto_https_setting = if enable_https { "" } else { "    auto_https off\n" };
    let caddyfile_content = format!(
        r#"{{
    admin 0.0.0.0:2019
{}}}

# Import all project configurations
import /etc/caddy/projects/*.caddy
"#,
        auto_https_setting
    );

    fs::write(&caddyfile_path, caddyfile_content)
        .context("Failed to write Caddyfile")?;

    println!("{} Caddyfile created", "✓".green());

    Ok(())
}

/// Ensure caddy-net network exists
fn ensure_caddy_network() -> Result<()> {
    let output = Command::new("docker")
        .args(&["network", "inspect", "caddy-net"])
        .output()
        .context("Failed to check network")?;

    if !output.status.success() {
        println!("{} Creating caddy-net network...", "ℹ".blue());
        let status = Command::new("docker")
            .args(&["network", "create", "caddy-net"])
            .status()
            .context("Failed to create network")?;

        if !status.success() {
            anyhow::bail!("Failed to create caddy-net network");
        }

        println!("{} Network created", "✓".green());
    }

    Ok(())
}

/// Start Caddy container
pub fn start() -> Result<()> {
    if is_running() {
        println!("{} Caddy is already running", "ℹ".blue());
        return Ok(());
    }

    // Check if stopped container exists
    if container_exists() {
        println!();
        println!("{} Found existing Caddy container (stopped)", "⚠".yellow());
        println!();
        println!("Choose an option:");
        println!("  1. {} - Start the existing container", "Start".green());
        println!("  2. {} - Remove and recreate container", "Reset".yellow());
        println!();
        print!("Enter choice (1 or 2): ");
        
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                start_existing_container()?;
                
                // Wait a bit for Caddy to start
                std::thread::sleep(std::time::Duration::from_secs(2));
                
                if is_running() {
                    println!("{}", "✓ Caddy started successfully".green());
                    println!();
                    println!("Caddy Admin API: http://localhost:2019");
                    println!("View logs: docker logs oh-my-dockers-caddy -f");
                } else {
                    println!("{}", "⚠ Caddy may have failed to start".yellow());
                    println!("Check logs: docker logs oh-my-dockers-caddy");
                }
                return Ok(());
            }
            "2" => {
                remove_container()?;
                // Continue to create new container below
            }
            _ => {
                anyhow::bail!("Invalid choice. Please enter 1 or 2.");
            }
        }
    }

    println!("{}", "Starting Caddy reverse proxy...".blue());

    // Ensure Caddyfile exists
    ensure_caddyfile()?;

    // Ensure network exists
    ensure_caddy_network()?;

    // Get config directory for volume mounts
    let config_dir = get_config_dir()?;
    let caddyfile_path = config_dir.join("caddy/Caddyfile");
    let certs_path = config_dir.join("caddy/certs");
    let projects_path = config_dir.join("caddy/projects");

    println!("{} Starting Caddy container...", "ℹ".blue());
    
    // Start Caddy using docker run
    let status = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name", "oh-my-dockers-caddy",
            "--restart", "unless-stopped",
            "-p", "80:80",
            "-p", "443:443",
            "-p", "443:443/udp",
            "-v", &format!("{}:/etc/caddy/Caddyfile:ro", caddyfile_path.display()),
            "-v", &format!("{}:/certs:ro", certs_path.display()),
            "-v", &format!("{}:/etc/caddy/projects:ro", projects_path.display()),
            "-v", "caddy_data:/data",
            "-v", "caddy_config:/config",
            "--network", "caddy-net",
            "-e", "CADDY_ADMIN=0.0.0.0:2019",
            "--label", "com.oh-my-dockers.service=caddy",
            "caddy:latest",
        ])
        .status()
        .context("Failed to start Caddy")?;

    if !status.success() {
        anyhow::bail!("Failed to start Caddy container");
    }

    // Wait a bit for Caddy to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    if is_running() {
        println!("{}", "✓ Caddy started successfully".green());
        println!();
        println!("Caddy Admin API: http://localhost:2019");
        println!("View logs: docker logs oh-my-dockers-caddy -f");
    } else {
        println!("{}", "⚠ Caddy may have failed to start".yellow());
        println!("Check logs: docker logs oh-my-dockers-caddy");
    }

    Ok(())
}

/// Stop Caddy container
pub fn stop() -> Result<()> {
    if !is_running() {
        println!("{} Caddy is not running", "ℹ".blue());
        return Ok(());
    }

    println!("{}", "Stopping Caddy...".blue());

    let status = Command::new("docker")
        .args(&["stop", "oh-my-dockers-caddy"])
        .status()
        .context("Failed to stop Caddy")?;

    if !status.success() {
        anyhow::bail!("Failed to stop Caddy");
    }

    println!("{}", "✓ Caddy stopped".green());

    Ok(())
}

/// Restart Caddy container
pub fn restart() -> Result<()> {
    if !is_running() {
        println!("{} Caddy is not running, starting it...", "ℹ".blue());
        return start();
    }

    println!("{}", "Restarting Caddy...".blue());

    let status = Command::new("docker")
        .args(&["restart", "oh-my-dockers-caddy"])
        .status()
        .context("Failed to restart Caddy")?;

    if !status.success() {
        anyhow::bail!("Failed to restart Caddy");
    }

    println!("{}", "✓ Caddy restarted".green());

    Ok(())
}

/// Show Caddy status
pub fn status() -> Result<()> {
    println!("{}", "Caddy Status:".blue());
    println!();

    if is_running() {
        println!("  Status: {}", "Running".green());

        // Get container details
        let output = Command::new("docker")
            .args(&[
                "ps",
                "--filter",
                "name=oh-my-dockers-caddy",
                "--format",
                "table {{.Status}}\t{{.Ports}}",
            ])
            .output()
            .context("Failed to get container status")?;

        let info = String::from_utf8_lossy(&output.stdout);
        for line in info.lines().skip(1) {
            println!("  {}", line);
        }

        println!();
        println!("Admin API: http://localhost:2019");
        println!("Logs: docker logs oh-my-dockers-caddy -f");
    } else {
        println!("  Status: {}", "Not running".red());
        println!();
        println!("Start Caddy with: {}", "omd caddy start".bright_white());
    }

    Ok(())
}

/// Show Caddy logs
pub fn logs(follow: bool) -> Result<()> {
    if !is_running() {
        println!("{} Caddy is not running", "⚠".yellow());
        return Ok(());
    }

    let mut args = vec!["logs"];
    if follow {
        args.push("-f");
    }
    args.push("oh-my-dockers-caddy");

    let status = Command::new("docker")
        .args(&args)
        .status()
        .context("Failed to show logs")?;

    if !status.success() {
        anyhow::bail!("Failed to show Caddy logs");
    }

    Ok(())
}

/// Auto-start Caddy if not running (called from project up)
pub fn auto_start_if_needed() -> Result<()> {
    if is_running() {
        return Ok(());
    }

    println!();
    println!("{} Caddy is not running", "ℹ".blue());
    println!("{} Starting Caddy automatically...", "ℹ".blue());
    println!();

    start()?;

    Ok(())
}

