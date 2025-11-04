use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result};
use dirs;

/// Get the configuration directory path
/// Checks OH_MY_DOCKERS_DIR environment variable first,
/// then defaults to ~/.oh-my-dockers
pub fn get_config_dir() -> Result<PathBuf> {
    if let Ok(custom_dir) = env::var("OH_MY_DOCKERS_DIR") {
        return Ok(PathBuf::from(custom_dir));
    }

    let home_dir = dirs::home_dir()
        .context("Failed to get home directory")?;
    
    Ok(home_dir.join(".oh-my-dockers"))
}

/// Ensure the configuration directory and all subdirectories exist
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;

    // Create main directory
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    // Create subdirectories
    let subdirs = [
        "projects",
        "caddy",
        "caddy/certs",
        "caddy/projects",
        "templates",
        "init",
        "generated",
    ];

    for subdir in &subdirs {
        let dir_path = config_dir.join(subdir);
        fs::create_dir_all(&dir_path)
            .context(format!("Failed to create subdirectory: {}", subdir))?;
    }

    // Create default config.toml if it doesn't exist
    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        create_default_config(&config_file)?;
    }

    Ok(config_dir)
}

fn create_default_config(config_path: &Path) -> Result<()> {
    let default_config = r#"# Global Configuration for oh-my-dockers

[global]
# Caddy network name
caddy_network = "caddy-net"

# Directories (relative to config directory)
projects_dir = "projects"
templates_dir = "templates"
init_dir = "init"
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
# Default versions for services
postgres_version = "latest"
redis_version = "latest"
surrealdb_version = "latest"
chroma_version = "latest"
ollama_version = "latest"
n8n_version = "latest"

# Default timezone
timezone = "Asia/Tokyo"
"#;

    fs::write(config_path, default_config)
        .context("Failed to write default config file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_config_dir_with_env_var() {
        env::set_var("OH_MY_DOCKERS_DIR", "/tmp/test-dir");
        let dir = get_config_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/test-dir"));
        env::remove_var("OH_MY_DOCKERS_DIR");
    }
}
