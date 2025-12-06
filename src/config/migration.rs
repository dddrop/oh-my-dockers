//! Configuration migration module
//!
//! This module handles automatic migration of config.toml when the
//! configuration structure changes between versions.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use toml::Value;

use super::CONFIG_VERSION;

/// Check if migration is needed and perform it if necessary
pub fn migrate_config_if_needed(config_path: &Path) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .context("Failed to read config.toml for migration check")?;

    // Parse as generic TOML value to check version
    let mut config: Value =
        toml::from_str(&content).context("Failed to parse config.toml for migration")?;

    // Get current version (default to 0 if not present)
    let current_version = config
        .get("version")
        .and_then(|v| v.as_integer())
        .unwrap_or(0) as u32;

    if current_version >= CONFIG_VERSION {
        // No migration needed
        return Ok(());
    }

    println!(
        "{} Config migration needed: v{} -> v{}",
        "ℹ".blue(),
        current_version,
        CONFIG_VERSION
    );

    // Backup before migration
    backup_config(config_path)?;

    // Apply migrations sequentially
    let mut version = current_version;
    while version < CONFIG_VERSION {
        migrate_from_version(&mut config, version)?;
        version += 1;
    }

    // Update version field
    config
        .as_table_mut()
        .unwrap()
        .insert("version".to_string(), Value::Integer(CONFIG_VERSION as i64));

    // Write migrated config
    let new_content = toml::to_string_pretty(&config).context("Failed to serialize config")?;

    // Add header comment
    let final_content = format!(
        "# Global Configuration for oh-my-dockers\n# DO NOT EDIT the version field manually\n{}",
        new_content
    );

    fs::write(config_path, final_content).context("Failed to write migrated config")?;

    println!(
        "{} Config migrated successfully to v{}",
        "✓".green(),
        CONFIG_VERSION
    );

    Ok(())
}

/// Backup the config file before migration
fn backup_config(config_path: &Path) -> Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!(
        "{}.backup.{}",
        config_path.file_name().unwrap().to_string_lossy(),
        timestamp
    );
    let backup_path = config_path.parent().unwrap().join(backup_name);

    fs::copy(config_path, &backup_path).context("Failed to create config backup")?;

    println!(
        "{} Config backup created: {}",
        "✓".green(),
        backup_path.display()
    );

    Ok(())
}

/// Apply migration from a specific version to the next version
fn migrate_from_version(config: &mut Value, from_version: u32) -> Result<()> {
    match from_version {
        0 => migrate_v0_to_v1(config)?,
        // Add more migrations here as needed:
        // 1 => migrate_v1_to_v2(config)?,
        // 2 => migrate_v2_to_v3(config)?,
        _ => {
            // Unknown version, skip
            println!(
                "{} Unknown config version {}, skipping migration",
                "⚠".yellow(),
                from_version
            );
        }
    }

    Ok(())
}

/// Migration from version 0 (no version field) to version 1
///
/// Changes:
/// - Add version field
/// - Ensure all required fields exist with defaults
fn migrate_v0_to_v1(config: &mut Value) -> Result<()> {
    println!("{} Migrating v0 -> v1: Adding version tracking", "ℹ".blue());

    let table = config.as_table_mut().context("Config is not a table")?;

    // Ensure [global] section exists
    if !table.contains_key("global") {
        let mut global = toml::map::Map::new();
        global.insert(
            "caddy_network".to_string(),
            Value::String("caddy-net".to_string()),
        );
        global.insert(
            "caddy_projects_dir".to_string(),
            Value::String("caddy/projects".to_string()),
        );
        global.insert(
            "caddy_certs_dir".to_string(),
            Value::String("caddy/certs".to_string()),
        );
        global.insert("enable_https".to_string(), Value::Boolean(true));
        table.insert("global".to_string(), Value::Table(global));
        println!("  {} Added [global] section with defaults", "+".green());
    } else {
        // Ensure all required fields in [global]
        if let Some(global) = table.get_mut("global").and_then(|v| v.as_table_mut()) {
            if !global.contains_key("caddy_network") {
                global.insert(
                    "caddy_network".to_string(),
                    Value::String("caddy-net".to_string()),
                );
                println!("  {} Added global.caddy_network", "+".green());
            }
            if !global.contains_key("caddy_projects_dir") {
                global.insert(
                    "caddy_projects_dir".to_string(),
                    Value::String("caddy/projects".to_string()),
                );
                println!("  {} Added global.caddy_projects_dir", "+".green());
            }
            if !global.contains_key("caddy_certs_dir") {
                global.insert(
                    "caddy_certs_dir".to_string(),
                    Value::String("caddy/certs".to_string()),
                );
                println!("  {} Added global.caddy_certs_dir", "+".green());
            }
            if !global.contains_key("enable_https") {
                global.insert("enable_https".to_string(), Value::Boolean(true));
                println!("  {} Added global.enable_https", "+".green());
            }
        }
    }

    // Ensure [defaults] section exists
    if !table.contains_key("defaults") {
        let mut defaults = toml::map::Map::new();
        defaults.insert(
            "timezone".to_string(),
            Value::String("Asia/Tokyo".to_string()),
        );
        table.insert("defaults".to_string(), Value::Table(defaults));
        println!("  {} Added [defaults] section", "+".green());
    }

    // Ensure [networks] section exists
    if !table.contains_key("networks") {
        let mut networks = toml::map::Map::new();
        networks.insert("caddy-net".to_string(), Value::Table(toml::map::Map::new()));
        table.insert("networks".to_string(), Value::Table(networks));
        println!("  {} Added [networks] section", "+".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_v0_to_v1_empty_config() {
        let mut config: Value = toml::from_str("").unwrap();
        migrate_v0_to_v1(&mut config).unwrap();

        assert!(config.get("global").is_some());
        assert!(config.get("defaults").is_some());
        assert!(config.get("networks").is_some());
    }

    #[test]
    fn test_migrate_v0_to_v1_partial_config() {
        let mut config: Value = toml::from_str(
            r#"
[global]
caddy_network = "my-net"
"#,
        )
        .unwrap();

        migrate_v0_to_v1(&mut config).unwrap();

        let global = config.get("global").unwrap();
        assert_eq!(
            global.get("caddy_network").unwrap().as_str().unwrap(),
            "my-net"
        );
        assert!(global.get("caddy_projects_dir").is_some());
        assert!(global.get("caddy_certs_dir").is_some());
        assert!(global.get("enable_https").is_some());
    }
}
