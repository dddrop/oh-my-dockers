use std::{collections::HashMap, process::Command};

use anyhow::{Context, Result};
use colored::Colorize;

#[derive(Debug, Clone)]
struct PortMapping {
    container: String,
    #[allow(dead_code)]
    network: String,
    internal_port: String,
    local_port: String,
    protocol: String,
}

/// List all port mappings across all networks
pub fn list() -> Result<()> {
    println!("{}", "Port Mappings:".blue());
    println!();

    // Get all running containers with port mappings
    let output = Command::new("docker")
        .args(&["ps", "--format", "{{.Names}}\t{{.Ports}}"])
        .output()
        .context("Failed to list containers")?;

    if !output.status.success() {
        anyhow::bail!("Failed to list containers");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        println!("{}", "No running containers found".yellow());
        return Ok(());
    }

    // Group by network
    let mut network_mappings: HashMap<String, Vec<PortMapping>> = HashMap::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let container = parts[0].to_string();
        let ports_str = parts[1];

        // Get networks for this container
        let networks_output = Command::new("docker")
            .args(&[
                "inspect",
                &container,
                "--format",
                "{{range $key, $value := .NetworkSettings.Networks}}{{$key}} {{end}}",
            ])
            .output()
            .context("Failed to inspect container")?;

        let networks = String::from_utf8_lossy(&networks_output.stdout);
        let network_list: Vec<&str> = networks.trim().split_whitespace().collect();

        // Parse port mappings
        if ports_str != "<none>" && !ports_str.is_empty() {
            let mappings = parse_port_mappings(ports_str)?;

            for mapping in mappings {
                // If container has multiple networks, add to each
                if network_list.is_empty() {
                    // No network info, use "unknown"
                    network_mappings
                        .entry("unknown".to_string())
                        .or_insert_with(Vec::new)
                        .push(PortMapping {
                            container: container.clone(),
                            network: "unknown".to_string(),
                            internal_port: mapping.internal_port,
                            local_port: mapping.local_port,
                            protocol: mapping.protocol,
                        });
                } else {
                    for network in &network_list {
                        network_mappings
                            .entry(network.to_string())
                            .or_insert_with(Vec::new)
                            .push(PortMapping {
                                container: container.clone(),
                                network: network.to_string(),
                                internal_port: mapping.internal_port.clone(),
                                local_port: mapping.local_port.clone(),
                                protocol: mapping.protocol.clone(),
                            });
                    }
                }
            }
        }
    }

    if network_mappings.is_empty() {
        println!("{}", "No port mappings found".yellow());
        return Ok(());
    }

    // Display by network
    let mut networks: Vec<&String> = network_mappings.keys().collect();
    networks.sort();

    for network in networks {
        println!("  {} {}", "Network:".bright_white(), network.bright_cyan());
        println!("  {}", "-".repeat(80));
        println!(
            "  {:<25} {:<15} {:<15} {:<10}",
            "CONTAINER", "INTERNAL", "LOCAL", "PROTOCOL"
        );
        println!("  {}", "-".repeat(80));

        let mappings = &network_mappings[network];
        for mapping in mappings {
            println!(
                "  {:<25} {:<15} {:<15} {:<10}",
                mapping.container.bright_white(),
                mapping.internal_port,
                mapping.local_port.bright_green(),
                mapping.protocol
            );
        }
        println!();
    }

    Ok(())
}

/// Show port mappings for a specific network
pub fn show(network: &str) -> Result<()> {
    println!(
        "{} Port Mappings for Network: {}",
        "".blue(),
        network.bright_cyan()
    );
    println!();

    // Get all containers in this network
    let output = Command::new("docker")
        .args(&[
            "network",
            "inspect",
            network,
            "--format",
            "{{range .Containers}}{{.Name}} {{end}}",
        ])
        .output()
        .context("Failed to inspect network")?;

    if !output.status.success() {
        anyhow::bail!("Network {} not found", network);
    }

    let containers_str = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<&str> = containers_str.trim().split_whitespace().collect();

    if containers.is_empty() {
        println!("{}", "No containers in this network".yellow());
        return Ok(());
    }

    println!(
        "  {:<25} {:<15} {:<15} {:<10}",
        "CONTAINER", "INTERNAL", "LOCAL", "PROTOCOL"
    );
    println!("  {}", "-".repeat(80));

    let mut found_any = false;

    for container in containers {
        // Get port mappings for this container
        let ps_output = Command::new("docker")
            .args(&[
                "ps",
                "--filter",
                &format!("name={}", container),
                "--format",
                "{{.Ports}}",
            ])
            .output()
            .context("Failed to get container ports")?;

        let ports_str = String::from_utf8_lossy(&ps_output.stdout)
            .trim()
            .to_string();

        if ports_str != "<none>" && !ports_str.is_empty() {
            let mappings = parse_port_mappings(&ports_str)?;
            for mapping in mappings {
                println!(
                    "  {:<25} {:<15} {:<15} {:<10}",
                    container.bright_white(),
                    mapping.internal_port,
                    mapping.local_port.bright_green(),
                    mapping.protocol
                );
                found_any = true;
            }
        }
    }

    if !found_any {
        println!(
            "{}",
            "No port mappings found for containers in this network".yellow()
        );
    }

    Ok(())
}

/// Parse port mappings from Docker ps output format
/// Format: "0.0.0.0:8080->80/tcp, 0.0.0.0:8443->443/tcp"
fn parse_port_mappings(ports_str: &str) -> Result<Vec<PortMapping>> {
    let mut mappings = Vec::new();

    // Split by comma for multiple mappings
    let parts: Vec<&str> = ports_str.split(',').collect();

    for part in parts {
        let part = part.trim();

        // Format: "0.0.0.0:8080->80/tcp" or "8080->80/tcp" or "80/tcp"
        if part.contains("->") {
            // Has local port mapping
            let arrow_parts: Vec<&str> = part.split("->").collect();
            if arrow_parts.len() == 2 {
                let local_part = arrow_parts[0].trim();
                let internal_part = arrow_parts[1].trim();

                // Extract local port (remove IP if present)
                let local_port = if local_part.contains(':') {
                    local_part.split(':').last().unwrap_or(local_part)
                } else {
                    local_part
                };

                // Extract internal port and protocol
                let internal_port = if internal_part.contains('/') {
                    internal_part.split('/').next().unwrap_or(internal_part)
                } else {
                    internal_part
                };

                let protocol = if internal_part.contains('/') {
                    internal_part.split('/').last().unwrap_or("tcp")
                } else {
                    "tcp"
                };

                mappings.push(PortMapping {
                    container: String::new(), // Will be filled by caller
                    network: String::new(),   // Will be filled by caller
                    internal_port: internal_port.to_string(),
                    local_port: local_port.to_string(),
                    protocol: protocol.to_string(),
                });
            }
        } else if part.contains('/') {
            // No local mapping, just internal port
            let port_parts: Vec<&str> = part.split('/').collect();
            if port_parts.len() >= 2 {
                mappings.push(PortMapping {
                    container: String::new(),
                    network: String::new(),
                    internal_port: port_parts[0].to_string(),
                    local_port: "<none>".to_string(),
                    protocol: port_parts[1].to_string(),
                });
            }
        }
    }

    Ok(mappings)
}
