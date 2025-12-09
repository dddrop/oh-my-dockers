//! Docker Compose file generator
//!
//! This module provides functionality to generate docker-compose.yml files
//! with pre-configured service templates.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use super::registry::PortRegistry;

/// Service template definition
#[derive(Debug, Clone)]
pub struct ServiceTemplate {
    pub name: &'static str,
    pub display_name: &'static str,
    pub image: &'static str,
    pub default_port: u16,
    pub container_port: u16,
    pub environment: &'static [(&'static str, &'static str)],
    pub volumes: &'static [&'static str],
}

/// Available service templates
pub const AVAILABLE_SERVICES: &[ServiceTemplate] = &[
    // PostgreSQL
    ServiceTemplate {
        name: "postgres",
        display_name: "PostgreSQL",
        image: "postgres:latest",
        default_port: 5432,
        container_port: 5432,
        environment: &[
            ("POSTGRES_USER", "postgres"),
            ("POSTGRES_PASSWORD", "postgres"),
            ("POSTGRES_DB", "app"),
        ],
        volumes: &["postgres_data:/var/lib/postgresql"],
    },
    // Redis
    ServiceTemplate {
        name: "redis",
        display_name: "Redis",
        image: "redis:latest",
        default_port: 6379,
        container_port: 6379,
        environment: &[],
        volumes: &["redis_data:/data"],
    },
    // Kafka (KRaft mode - no Zookeeper required)
    ServiceTemplate {
        name: "kafka",
        display_name: "Kafka (KRaft)",
        image: "apache/kafka:latest",
        default_port: 9092,
        container_port: 9092,
        environment: &[
            ("KAFKA_NODE_ID", "1"),
            ("KAFKA_PROCESS_ROLES", "broker,controller"),
            ("KAFKA_LISTENERS", "PLAINTEXT://:9092,CONTROLLER://:9093"),
            ("KAFKA_ADVERTISED_LISTENERS", "PLAINTEXT://localhost:9092"),
            ("KAFKA_CONTROLLER_LISTENER_NAMES", "CONTROLLER"),
            ("KAFKA_CONTROLLER_QUORUM_VOTERS", "1@kafka:9093"),
            ("KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR", "1"),
            ("CLUSTER_ID", "omd-kafka-cluster-id-001"),
        ],
        volumes: &["kafka_data:/var/lib/kafka/data"],
    },
];

/// Find an available port that doesn't conflict with existing ports
pub fn find_available_port(desired: u16, used_ports: &[u16]) -> u16 {
    let mut port = desired;
    while used_ports.contains(&port) {
        port += 1;
    }
    port
}

/// Selected service with resolved port
#[derive(Debug, Clone)]
pub struct SelectedService {
    pub template: &'static ServiceTemplate,
    pub host_port: u16,
}

/// Prompt user to select services from available templates
pub fn prompt_service_selection() -> Result<Vec<usize>> {
    println!();
    println!(
        "{}",
        "Select services (enter numbers separated by space, empty to skip):".blue()
    );
    println!();

    for (idx, service) in AVAILABLE_SERVICES.iter().enumerate() {
        println!(
            "  [{}] {} (port {})",
            idx + 1,
            service.display_name.bright_white(),
            service.default_port
        );
    }

    println!();
    print!("Your selection: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selections: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|&n| n >= 1 && n <= AVAILABLE_SERVICES.len())
        .map(|n| n - 1) // Convert to 0-based index
        .collect();

    Ok(selections)
}

/// Resolve ports for selected services, avoiding conflicts
pub fn resolve_service_ports(
    selections: &[usize],
    registry: &PortRegistry,
) -> Vec<SelectedService> {
    let mut used_ports = registry.get_all_used_ports();
    let mut selected_services = Vec::new();

    println!();
    println!("{} Checking port conflicts...", "ℹ".blue());

    for &idx in selections {
        let template = &AVAILABLE_SERVICES[idx];
        let desired_port = template.default_port;
        let host_port = find_available_port(desired_port, &used_ports);

        if host_port != desired_port {
            println!(
                "{} Port {} in use, using {} for {}",
                "⚠".yellow(),
                desired_port,
                host_port.to_string().green(),
                template.display_name
            );
        } else {
            println!(
                "{} Port {} available for {}",
                "✓".green(),
                host_port,
                template.display_name
            );
        }

        // Add to used ports to avoid conflicts between selected services
        used_ports.push(host_port);

        selected_services.push(SelectedService {
            template,
            host_port,
        });
    }

    selected_services
}

/// Generate docker-compose.yml content
pub fn generate_compose_content(
    project_name: &str,
    network_name: &str,
    services: &[SelectedService],
) -> String {
    let mut content = String::from("# Generated by oh-my-dockers\n");
    content.push_str("services:\n");

    for service in services {
        content.push_str(&generate_service_block(project_name, network_name, service));
    }

    // Generate volumes section
    if !services.is_empty() {
        content.push_str("\nvolumes:\n");
        for service in services {
            for volume in service.template.volumes {
                if let Some(volume_name) = volume.split(':').next() {
                    content.push_str(&format!("  {}:\n", volume_name));
                }
            }
        }
    }

    // Generate networks section
    content.push_str(&format!("\nnetworks:\n  {}:\n", network_name));

    content
}

/// Generate a single service block
fn generate_service_block(
    project_name: &str,
    network_name: &str,
    service: &SelectedService,
) -> String {
    let template = service.template;
    let mut block = format!("  {}:\n", template.name);

    // Image
    block.push_str(&format!("    image: {}\n", template.image));

    // Container name
    block.push_str(&format!(
        "    container_name: {}-{}\n",
        project_name, template.name
    ));

    // Restart policy
    block.push_str("    restart: unless-stopped\n");

    // Ports
    block.push_str("    ports:\n");
    block.push_str(&format!(
        "      - \"{}:{}\"\n",
        service.host_port, template.container_port
    ));

    // Environment variables
    if !template.environment.is_empty() {
        block.push_str("    environment:\n");
        for (key, value) in template.environment {
            block.push_str(&format!("      {}: {}\n", key, value));
        }
    }

    // Volumes
    if !template.volumes.is_empty() {
        block.push_str("    volumes:\n");
        for volume in template.volumes {
            block.push_str(&format!("      - {}\n", volume));
        }
    }

    // Networks
    block.push_str("    networks:\n");
    block.push_str(&format!("      - {}\n", network_name));

    block.push('\n');
    block
}

/// Generate docker-compose.yml file
pub fn generate_compose_file(
    path: &Path,
    project_name: &str,
    network_name: &str,
    services: &[SelectedService],
) -> Result<()> {
    let content = generate_compose_content(project_name, network_name, services);
    fs::write(path, content).context("Failed to write docker-compose.yml")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let used = vec![5432, 5433, 6379];

        assert_eq!(find_available_port(5432, &used), 5434);
        assert_eq!(find_available_port(6380, &used), 6380);
        assert_eq!(find_available_port(6379, &used), 6380);
    }

    #[test]
    fn test_generate_compose_content() {
        let services = vec![SelectedService {
            template: &AVAILABLE_SERVICES[0], // PostgreSQL
            host_port: 5432,
        }];

        let content = generate_compose_content("myproject", "myproject-net", &services);

        assert!(content.contains("postgres:latest"));
        assert!(content.contains("myproject-postgres"));
        assert!(content.contains("5432:5432"));
        assert!(content.contains("POSTGRES_USER"));
    }
}
