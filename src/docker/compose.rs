//! Docker Compose file parsing
//!
//! This module handles parsing docker-compose.yml files to extract
//! service information, port mappings, and network configurations.

use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};
use serde_yaml::Value;

/// Information extracted from a docker-compose.yml file
#[derive(Debug, Clone)]
pub struct ComposeInfo {
    /// Service name -> ServiceInfo
    pub services: HashMap<String, ServiceInfo>,
}

/// Information about a single service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Container name (if specified, otherwise generated)
    pub container_name: Option<String>,
    /// Host ports mapped (port mappings like "8080:80" -> 8080)
    pub host_ports: Vec<u16>,
    /// Container ports (port mappings like "8080:80" -> 80)
    pub container_ports: Vec<u16>,
    /// Networks this service is connected to
    #[allow(dead_code)]
    pub networks: Vec<String>,
}

impl ComposeInfo {
    /// Parse a docker-compose.yml file
    pub fn parse(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read docker-compose file: {:?}", path))?;

        let yaml: Value =
            serde_yaml::from_str(&content).context("Failed to parse docker-compose YAML")?;

        let mut services = HashMap::new();

        if let Some(services_map) = yaml.get("services").and_then(|v| v.as_mapping()) {
            for (service_name, service_config) in services_map {
                let name = service_name
                    .as_str()
                    .context("Service name is not a string")?
                    .to_string();

                let container_name = service_config
                    .get("container_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let (host_ports, container_ports) = Self::parse_ports(service_config)?;

                let networks = Self::parse_networks(service_config);

                let service_info = ServiceInfo {
                    name: name.clone(),
                    container_name,
                    host_ports,
                    container_ports,
                    networks,
                };

                services.insert(name, service_info);
            }
        }

        Ok(Self { services })
    }

    /// Parse port mappings from a service configuration
    fn parse_ports(service_config: &Value) -> Result<(Vec<u16>, Vec<u16>)> {
        let mut host_ports = Vec::new();
        let mut container_ports = Vec::new();

        if let Some(ports) = service_config.get("ports").and_then(|v| v.as_sequence()) {
            for port_entry in ports {
                if let Some(port_str) = port_entry.as_str() {
                    // Handle "host:container" format
                    if let Some((host, container)) = port_str.split_once(':') {
                        // Handle "host_ip:host_port:container_port" format
                        let (host_port_str, container_port_str) = if container.contains(':') {
                            let parts: Vec<&str> = port_str.split(':').collect();
                            if parts.len() == 3 {
                                (parts[1], parts[2])
                            } else {
                                continue;
                            }
                        } else {
                            (host, container)
                        };

                        // Parse host port (may have range like "8080-8090")
                        if let Some((start, end)) = host_port_str.split_once('-') {
                            if let (Ok(start_port), Ok(end_port)) =
                                (start.parse::<u16>(), end.parse::<u16>())
                            {
                                for port in start_port..=end_port {
                                    host_ports.push(port);
                                }
                            }
                        } else if let Ok(port) = host_port_str.parse::<u16>() {
                            host_ports.push(port);
                        }

                        // Parse container port
                        let container_port_only = container_port_str
                            .split('/')
                            .next()
                            .unwrap_or(container_port_str);
                        if let Ok(port) = container_port_only.parse::<u16>() {
                            container_ports.push(port);
                        }
                    } else {
                        // Handle single port (only container port, no host mapping)
                        let port_only = port_str.split('/').next().unwrap_or(port_str);
                        if let Ok(port) = port_only.parse::<u16>() {
                            container_ports.push(port);
                        }
                    }
                } else if let Some(port_obj) = port_entry.as_mapping() {
                    // Handle long syntax
                    if let Some(published) = port_obj.get(&Value::String("published".to_string())) {
                        if let Some(port) = published.as_u64() {
                            host_ports.push(port as u16);
                        } else if let Some(port_str) = published.as_str() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                host_ports.push(port);
                            }
                        }
                    }
                    if let Some(target) = port_obj.get(&Value::String("target".to_string())) {
                        if let Some(port) = target.as_u64() {
                            container_ports.push(port as u16);
                        } else if let Some(port_str) = target.as_str() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                container_ports.push(port);
                            }
                        }
                    }
                }
            }
        }

        Ok((host_ports, container_ports))
    }

    /// Parse networks from a service configuration
    fn parse_networks(service_config: &Value) -> Vec<String> {
        let mut networks = Vec::new();

        if let Some(networks_value) = service_config.get("networks") {
            if let Some(networks_seq) = networks_value.as_sequence() {
                // Array format: ["network1", "network2"]
                for network in networks_seq {
                    if let Some(network_str) = network.as_str() {
                        networks.push(network_str.to_string());
                    }
                }
            } else if let Some(networks_map) = networks_value.as_mapping() {
                // Map format: { network1: {}, network2: {} }
                for (network_name, _) in networks_map {
                    if let Some(network_str) = network_name.as_str() {
                        networks.push(network_str.to_string());
                    }
                }
            }
        }

        networks
    }

    /// Get all host ports used across all services
    pub fn get_all_host_ports(&self) -> Vec<u16> {
        let mut all_ports = Vec::new();
        for service in self.services.values() {
            all_ports.extend(&service.host_ports);
        }
        all_ports.sort();
        all_ports.dedup();
        all_ports
    }

    /// Get all container names (generated or explicit)
    pub fn get_all_container_names(&self, project_name: &str) -> Vec<String> {
        self.services
            .values()
            .map(|service| {
                service.container_name.clone().unwrap_or_else(|| {
                    // Default Docker Compose naming: project_service_1
                    format!("{}-{}-1", project_name, service.name)
                })
            })
            .collect()
    }

    /// Get services that are on a specific network
    #[allow(dead_code)]
    pub fn get_services_on_network(&self, network_name: &str) -> Vec<&ServiceInfo> {
        self.services
            .values()
            .filter(|service| service.networks.contains(&network_name.to_string()))
            .collect()
    }
}

/// Ensure the network in docker-compose.yml is marked as external.
/// This prevents Docker Compose from creating a new network with a project prefix.
pub fn ensure_network_external(path: &Path, network_name: &str) -> Result<bool> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read docker-compose file: {:?}", path))?;

    let mut yaml: Value =
        serde_yaml::from_str(&content).context("Failed to parse docker-compose YAML")?;

    let mut modified = false;

    if let Some(networks) = yaml.get_mut("networks").and_then(|v| v.as_mapping_mut()) {
        let network_key = Value::String(network_name.to_string());

        if let Some(network_config) = networks.get_mut(&network_key) {
            let external_key = Value::String("external".to_string());

            // Check if external is already set to true
            let is_external = network_config
                .get(&external_key)
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !is_external {
                // Convert null/empty to mapping if needed
                if network_config.is_null() {
                    *network_config = Value::Mapping(serde_yaml::Mapping::new());
                }

                if let Some(mapping) = network_config.as_mapping_mut() {
                    mapping.insert(external_key, Value::Bool(true));
                    modified = true;
                }
            }
        }
    }

    if modified {
        let new_content =
            serde_yaml::to_string(&yaml).context("Failed to serialize docker-compose YAML")?;
        fs::write(path, new_content)
            .context(format!("Failed to write docker-compose file: {:?}", path))?;
    }

    Ok(modified)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_parse_simple_compose() {
        let yaml = r#"
services:
  postgres:
    image: postgres:latest
    container_name: my-postgres
    ports:
      - "5432:5432"
    networks:
      - mynet

  redis:
    image: redis:latest
    ports:
      - "6379:6379"
    networks:
      - mynet

networks:
  mynet:
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let info = ComposeInfo::parse(file.path()).unwrap();

        assert_eq!(info.services.len(), 2);

        let postgres = info.services.get("postgres").unwrap();
        assert_eq!(postgres.container_name, Some("my-postgres".to_string()));
        assert_eq!(postgres.host_ports, vec![5432]);
        assert_eq!(postgres.container_ports, vec![5432]);
        assert_eq!(postgres.networks, vec!["mynet"]);

        let redis = info.services.get("redis").unwrap();
        assert_eq!(redis.container_name, None);
        assert_eq!(redis.host_ports, vec![6379]);
    }

    #[test]
    fn test_parse_port_ranges() {
        let yaml = r#"
services:
  app:
    image: app:latest
    ports:
      - "8080-8085:8080"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let info = ComposeInfo::parse(file.path()).unwrap();
        let app = info.services.get("app").unwrap();

        assert_eq!(app.host_ports.len(), 6);
        assert!(app.host_ports.contains(&8080));
        assert!(app.host_ports.contains(&8085));
    }

    #[test]
    fn test_parse_long_syntax_ports() {
        let yaml = r#"
services:
  app:
    image: app:latest
    ports:
      - target: 80
        published: 8080
        protocol: tcp
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let info = ComposeInfo::parse(file.path()).unwrap();
        let app = info.services.get("app").unwrap();

        assert_eq!(app.host_ports, vec![8080]);
        assert_eq!(app.container_ports, vec![80]);
    }

    #[test]
    fn test_ensure_network_external_updates_non_external() {
        let yaml = r#"
services:
  app:
    image: app:latest
    networks:
      - mynet

networks:
  mynet:
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let modified = ensure_network_external(file.path(), "mynet").unwrap();
        assert!(modified);

        // Verify the file was updated
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("external: true") || content.contains("external:true"));
    }

    #[test]
    fn test_ensure_network_external_already_external() {
        let yaml = r#"
services:
  app:
    image: app:latest
    networks:
      - mynet

networks:
  mynet:
    external: true
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let modified = ensure_network_external(file.path(), "mynet").unwrap();
        assert!(!modified);
    }

    #[test]
    fn test_ensure_network_external_unknown_network() {
        let yaml = r#"
services:
  app:
    image: app:latest

networks:
  other-net:
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let modified = ensure_network_external(file.path(), "mynet").unwrap();
        assert!(!modified);
    }
}
