use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{get_config_dir, load_global_config, ProjectConfig};

#[derive(Debug)]
struct TemplateContent {
    services: String,
    volumes: String,
}

pub fn generate_compose_file(
    project: &str,
    config: &ProjectConfig,
    env_vars: &HashMap<String, String>,
) -> Result<String> {
    println!("{} Generating docker-compose file...", "ℹ".blue());

    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let output_dir = config_dir.join("generated");
    fs::create_dir_all(&output_dir)
        .context("Failed to create generated directory")?;

    let output_file = output_dir.join(format!("docker-compose-{}.yml", project));

    // Prepare environment variables with defaults
    let mut all_env = HashMap::new();
    all_env.insert("PROJECT_NAME".to_string(), config.project.name.clone());
    all_env.insert("PROJECT_DOMAIN".to_string(), config.project.domain.clone());
    all_env.insert("PROJECT_NETWORK".to_string(), config.network.name.clone());

    // Use absolute path for init dir
    let init_dir = config_dir
        .join(&global_config.global.init_dir)
        .to_string_lossy()
        .to_string();
    all_env.insert("INIT_DIR".to_string(), init_dir);

    // Add port offset for database services
    // Validate that offset doesn't cause integer overflow or exceed max port (65535)
    let offset = config.project.port_offset;
    
    // Base ports: PostgreSQL=5432, Redis=6379, MySQL=3306, MongoDB=27017
    // Max port is 65535, so validate each calculation won't overflow or exceed max
    let postgres_port = 5432u32
        .checked_add(offset as u32)
        .and_then(|p| if p <= 65535 { Some(p) } else { None })
        .ok_or_else(|| anyhow::anyhow!("Port offset {} would cause overflow or exceed max port for PostgreSQL (5432 + {} > 65535)", offset, offset))?;
    
    let redis_port = 6379u32
        .checked_add(offset as u32)
        .and_then(|p| if p <= 65535 { Some(p) } else { None })
        .ok_or_else(|| anyhow::anyhow!("Port offset {} would cause overflow or exceed max port for Redis (6379 + {} > 65535)", offset, offset))?;
    
    let mysql_port = 3306u32
        .checked_add(offset as u32)
        .and_then(|p| if p <= 65535 { Some(p) } else { None })
        .ok_or_else(|| anyhow::anyhow!("Port offset {} would cause overflow or exceed max port for MySQL (3306 + {} > 65535)", offset, offset))?;
    
    let mongodb_port = 27017u32
        .checked_add(offset as u32)
        .and_then(|p| if p <= 65535 { Some(p) } else { None })
        .ok_or_else(|| anyhow::anyhow!("Port offset {} would cause overflow or exceed max port for MongoDB (27017 + {} > 65535)", offset, offset))?;
    
    all_env.insert("POSTGRES_PORT".to_string(), postgres_port.to_string());
    all_env.insert("REDIS_PORT".to_string(), redis_port.to_string());
    all_env.insert("MYSQL_PORT".to_string(), mysql_port.to_string());
    all_env.insert("MONGODB_PORT".to_string(), mongodb_port.to_string());

    // Add user-provided env vars
    for (k, v) in env_vars {
        all_env.insert(k.clone(), v.clone());
    }

    // Parse and collect all template parts
    let mut services_parts = Vec::new();
    let mut volumes_parts = Vec::new();

    let templates_dir = config_dir.join(&global_config.global.templates_dir);

    // Add enabled services
    for (service_name, service_config) in &config.services {
        if !service_config.enabled {
            continue;
        }

        let template_path = templates_dir.join(format!("{}.yml", service_name));
        if !template_path.exists() {
            println!(
                "{} Template not found: {:?} (skipping)",
                "⚠".yellow(),
                template_path
            );
            continue;
        }

        println!("{} Adding service: {}", "ℹ".blue(), service_name);

        let template = fs::read_to_string(&template_path)
            .context(format!("Failed to read template: {:?}", template_path))?;

        // Replace environment variables in template
        let processed = replace_env_vars(
            &template,
            &all_env,
            service_name,
            service_config.version.as_deref(),
        );

        // Parse template into sections
        let parsed = parse_template(&processed);
        services_parts.push(parsed.services);
        if !parsed.volumes.is_empty() {
            volumes_parts.push(parsed.volumes);
        }
    }

    // Build final compose file
    let mut compose_content = format!(
        "# Auto-generated docker-compose file for {}\n# Generated at: {}\n\nname: oh-my-dockers\n\n",
        project,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    // Add services section
    compose_content.push_str("services:\n");
    for service_part in services_parts {
        compose_content.push_str(&service_part);
        compose_content.push('\n');
    }

    // Add volumes section
    if !volumes_parts.is_empty() {
        compose_content.push_str("\nvolumes:\n");
        for volume_part in volumes_parts {
            compose_content.push_str(&volume_part);
        }
    }

    // Add networks section
    compose_content.push_str("\nnetworks:\n");
    compose_content.push_str(&format!("  {}:\n", config.network.name));
    compose_content.push_str("    external: true\n");
    compose_content.push_str(&format!("  {}:\n", global_config.global.caddy_network));
    compose_content.push_str("    external: true\n");

    fs::write(&output_file, compose_content)
        .context("Failed to write compose file")?;

    println!("{} Generated {:?}", "✓".green(), output_file);

    Ok(output_file.to_string_lossy().to_string())
}

fn parse_template(content: &str) -> TemplateContent {
    let lines: Vec<&str> = content.lines().collect();
    let mut services = String::new();
    let mut volumes = String::new();

    let mut _in_services_section = false;
    let mut _in_volumes_section = false;

    for line in lines {
        let trimmed = line.trim();

        // Detect top-level section headers (no indentation)
        // Skip empty lines to avoid resetting section flags
        if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            if trimmed == "services:" {
                _in_services_section = true;
                _in_volumes_section = false;
                continue;
            } else if trimmed == "volumes:" {
                _in_services_section = false;
                _in_volumes_section = true;
                continue;
            } else if trimmed == "networks:" {
                // Skip networks section from templates
                _in_services_section = false;
                _in_volumes_section = false;
                break;
            } else {
                // Unrecognized non-indented line (e.g., comments, unknown sections)
                // Reset section flags to avoid incorrectly adding content
                _in_services_section = false;
                _in_volumes_section = false;
                continue;
            }
        }

        // Add content to appropriate section
        // Only add properly indented lines (not empty lines or non-indented content)
        if _in_services_section && !line.is_empty() {
            services.push_str(line);
            services.push('\n');
        } else if _in_volumes_section && !line.is_empty() {
            // Only include top-level volume definitions (2 spaces indent)
            // Skip nested volume lists (those with '-' are mount points inside services)
            if line.starts_with("  ") && !trimmed.starts_with('-') {
                volumes.push_str(line);
                volumes.push('\n');
            }
        }
    }

    TemplateContent { services, volumes }
}

fn replace_env_vars(
    template: &str,
    env_vars: &HashMap<String, String>,
    service_name: &str,
    version: Option<&str>,
) -> String {
    let mut result = template.to_string();

    // Replace all ${VAR} and ${VAR:-default} patterns
    for (key, value) in env_vars {
        let patterns = [format!("${{{}}}", key), format!("${{{}:-", key)];

        for pattern in &patterns {
            if result.contains(pattern) {
                if pattern.ends_with(":-") {
                    // Handle ${VAR:-default} pattern
                    let re_pattern = format!(r"\$\{{{}\:\-([^}}]+)\}}", regex::escape(key));
                    if let Ok(re) = regex::Regex::new(&re_pattern) {
                        result = re.replace_all(&result, value.as_str()).to_string();
                    }
                } else {
                    result = result.replace(pattern, value);
                }
            }
        }
    }

    // Handle version variable if provided
    if let Some(ver) = version {
        let version_key = format!("{}_VERSION", service_name.to_uppercase());
        result = result.replace(&format!("${{{}}}", version_key), ver);
        result = result.replace(&format!("${{{}:-latest}}", version_key), ver);
    }

    // Replace any remaining ${VAR:-default} with default
    let re = regex::Regex::new(r"\$\{[^:}]+:-([^}]+)\}").unwrap();
    result = re.replace_all(&result, "$1").to_string();

    result
}

