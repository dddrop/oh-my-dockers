use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::ProjectConfig;

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

    let output_dir = Path::new(".generated");
    fs::create_dir_all(output_dir).context("Failed to create .generated directory")?;

    let output_file = format!(".generated/docker-compose-{}.yml", project);

    // Prepare environment variables with defaults
    let mut all_env = HashMap::new();
    all_env.insert("PROJECT_NAME".to_string(), config.project.name.clone());
    all_env.insert("PROJECT_DOMAIN".to_string(), config.project.domain.clone());
    all_env.insert("PROJECT_NETWORK".to_string(), config.network.name.clone());

    // Use absolute path for init dir to avoid relative path issues with .generated/
    let init_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("init")
        .to_string_lossy()
        .to_string();
    all_env.insert("INIT_DIR".to_string(), init_dir);

    // Add port offset for database services
    let offset = config.project.port_offset;
    all_env.insert("POSTGRES_PORT".to_string(), (5432 + offset).to_string());
    all_env.insert("REDIS_PORT".to_string(), (6379 + offset).to_string());
    all_env.insert("MYSQL_PORT".to_string(), (3306 + offset).to_string());
    all_env.insert("MONGODB_PORT".to_string(), (27017 + offset).to_string());

    // Add user-provided env vars
    for (k, v) in env_vars {
        all_env.insert(k.clone(), v.clone());
    }

    // Parse and collect all template parts
    let mut services_parts = Vec::new();
    let mut volumes_parts = Vec::new();

    // Add enabled services
    for (service_name, service_config) in &config.services {
        if !service_config.enabled {
            continue;
        }

        let template_path = format!("templates/{}.yml", service_name);
        if !Path::new(&template_path).exists() {
            println!(
                "{} Template not found: {} (skipping)",
                "⚠".yellow(),
                template_path
            );
            continue;
        }

        println!("{} Adding service: {}", "ℹ".blue(), service_name);

        let template = fs::read_to_string(&template_path)
            .context(format!("Failed to read template: {}", template_path))?;

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
    compose_content.push_str("  caddy-net:\n");
    compose_content.push_str("    external: true\n");

    fs::write(&output_file, compose_content).context("Failed to write compose file")?;

    println!("{} Generated {}", "✓".green(), output_file);

    Ok(output_file)
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
        if !line.starts_with(' ') && !line.starts_with('\t') {
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
            }
        }

        // Add content to appropriate section
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
