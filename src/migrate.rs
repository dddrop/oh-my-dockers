use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::ensure_config_dir;

/// Migrate existing configuration files to the new config directory
pub fn migrate_from_current_dir() -> Result<()> {
    println!("{} Starting migration...", "ℹ".blue());
    println!();

    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;

    let config_dir = ensure_config_dir()?;

    // Check if there's anything to migrate
    let projects_dir = current_dir.join("projects");
    let templates_dir = current_dir.join("templates");
    let init_dir = current_dir.join("init");
    let caddy_dir = current_dir.join("caddy");
    let config_file = current_dir.join("config.toml");

    let mut found_anything = false;

    // Migrate projects
    if projects_dir.exists() {
        println!("{} Migrating projects...", "ℹ".blue());
        let target_projects_dir = config_dir.join("projects");
        copy_directory(&projects_dir, &target_projects_dir)?;
        found_anything = true;
        println!("{} Projects migrated", "✓".green());
    }

    // Migrate templates
    if templates_dir.exists() {
        println!("{} Migrating templates...", "ℹ".blue());
        let target_templates_dir = config_dir.join("templates");
        copy_directory(&templates_dir, &target_templates_dir)?;
        found_anything = true;
        println!("{} Templates migrated", "✓".green());
    }

    // Migrate init scripts
    if init_dir.exists() {
        println!("{} Migrating init scripts...", "ℹ".blue());
        let target_init_dir = config_dir.join("init");
        copy_directory(&init_dir, &target_init_dir)?;
        found_anything = true;
        println!("{} Init scripts migrated", "✓".green());
    }

    // Migrate Caddy configuration
    if caddy_dir.exists() {
        println!("{} Migrating Caddy configuration...", "ℹ".blue());
        let target_caddy_dir = config_dir.join("caddy");
        
        // Copy Caddyfile
        let caddyfile = caddy_dir.join("Caddyfile");
        if caddyfile.exists() {
            let target_caddyfile = target_caddy_dir.join("Caddyfile");
            fs::copy(&caddyfile, &target_caddyfile)
                .context("Failed to copy Caddyfile")?;
            println!("  {} Caddyfile migrated", "✓".green());
        }

        // Copy certs directory
        let certs_dir = caddy_dir.join("certs");
        if certs_dir.exists() {
            let target_certs_dir = target_caddy_dir.join("certs");
            copy_directory(&certs_dir, &target_certs_dir)?;
            println!("  {} Certificates migrated", "✓".green());
        }

        // Copy projects directory
        let caddy_projects_dir = caddy_dir.join("projects");
        if caddy_projects_dir.exists() {
            let target_caddy_projects_dir = target_caddy_dir.join("projects");
            copy_directory(&caddy_projects_dir, &target_caddy_projects_dir)?;
            println!("  {} Caddy project configs migrated", "✓".green());
        }

        found_anything = true;
    }

    // Migrate global config.toml (update paths if needed)
    if config_file.exists() {
        println!("{} Checking global config...", "ℹ".blue());
        let target_config_file = config_dir.join("config.toml");
        
        // Only migrate if target doesn't exist or is empty
        if !target_config_file.exists() || fs::metadata(&target_config_file)?.len() == 0 {
            let content = fs::read_to_string(&config_file)
                .context("Failed to read config.toml")?;
            
            // Update paths in config if needed
            let updated_content = update_config_paths(&content);
            
            fs::write(&target_config_file, updated_content)
                .context("Failed to write config.toml")?;
            println!("{} Global config migrated", "✓".green());
            found_anything = true;
        } else {
            println!("{} Global config already exists, skipping", "ℹ".blue());
        }
    }

    if !found_anything {
        println!("{} No configuration files found to migrate", "⚠".yellow());
        return Ok(());
    }

    println!();
    println!(
        "{} Migration completed! Configuration is now in: {:?}",
        "✓".green(),
        config_dir
    );
    println!();
    println!("You can now use the new CLI tool:");
    println!("  oh-my-dockers project list");
    println!("  oh-my-dockers network list");
    println!("  oh-my-dockers ports list");

    Ok(())
}

/// Copy directory recursively
fn copy_directory(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }

    fs::create_dir_all(target)
        .context(format!("Failed to create target directory: {:?}", target))?;

    let entries = fs::read_dir(source)
        .context(format!("Failed to read source directory: {:?}", source))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let target_path = target.join(file_name);

        if path.is_dir() {
            copy_directory(&path, &target_path)?;
        } else {
            fs::copy(&path, &target_path)
                .context(format!("Failed to copy file: {:?}", path))?;
        }
    }

    Ok(())
}

/// Update paths in config.toml to be relative to config directory
fn update_config_paths(content: &str) -> String {
    let mut result = content.to_string();
    
    // Update paths to be relative
    result = result.replace("./projects", "projects");
    result = result.replace("./templates", "templates");
    result = result.replace("./init", "init");
    result = result.replace("./caddy/projects", "caddy/projects");
    result = result.replace("./caddy/certs", "caddy/certs");
    
    result
}

