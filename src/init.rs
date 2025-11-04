use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::get_current_dir_name;

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

[network]
# Docker network name for this project
name = "{}"
"#,
        project_name, domain, network
    );

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
    fs::write(config_path, config_content)
        .context("Failed to write omd.toml")?;

    println!();
    println!("{} Created {}", "✓".green(), "omd.toml".bright_white());
    println!();
    println!("Next steps:");
    println!("  1. Create or update your docker-compose.yml");
    println!("  2. Run {} to configure Caddy and check for port conflicts", "omd up".bright_white());
    println!("  3. Run {} to start your services", "docker compose up -d".bright_white());

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

