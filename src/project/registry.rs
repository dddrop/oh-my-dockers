//! Project registry
//!
//! This module manages the registry of projects and their port allocations.
//! The registry is stored as a JSON file in the configuration directory.

use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::get_config_dir;

/// Represents a registered project with its port allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    /// Project name
    pub name: String,
    /// Absolute path to the project directory
    pub path: PathBuf,
    /// Project domain
    pub domain: String,
    /// Network name
    pub network: String,
    /// List of host ports this project occupies
    pub ports: Vec<u16>,
    /// List of container names
    pub containers: Vec<String>,
}

/// Port registry that tracks all registered projects
#[derive(Debug, Serialize, Deserialize)]
pub struct PortRegistry {
    /// Map of project name to project entry
    projects: HashMap<String, ProjectEntry>,
}

impl PortRegistry {
    /// Create a new empty port registry
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    /// Load the port registry from disk
    pub fn load() -> Result<Self> {
        let registry_path = Self::get_registry_path()?;

        if !registry_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&registry_path).context("Failed to read registry file")?;

        serde_json::from_str(&content).context("Failed to parse registry file")
    }

    /// Save the port registry to disk
    pub fn save(&self) -> Result<()> {
        let registry_path = Self::get_registry_path()?;

        let content = serde_json::to_string_pretty(self).context("Failed to serialize registry")?;

        fs::write(&registry_path, content).context("Failed to write registry file")?;

        Ok(())
    }

    /// Get the path to the registry file
    fn get_registry_path() -> Result<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("registry.json"))
    }

    /// Register a new project or update an existing one
    pub fn register_project(&mut self, entry: ProjectEntry) -> Result<()> {
        self.projects.insert(entry.name.clone(), entry);
        self.save()
    }

    /// Unregister a project by name
    pub fn unregister_project(&mut self, project_name: &str) -> Result<()> {
        self.projects.remove(project_name);
        self.save()
    }

    /// Check for port conflicts
    /// Returns a list of (conflicting_port, conflicting_project_name) tuples
    pub fn check_port_conflicts(&self, project_name: &str, ports: &[u16]) -> Vec<(u16, String)> {
        let mut conflicts = Vec::new();

        for port in ports {
            for (name, entry) in &self.projects {
                // Skip the project itself (for updates)
                if name == project_name {
                    continue;
                }

                if entry.ports.contains(port) {
                    conflicts.push((*port, name.clone()));
                }
            }
        }

        conflicts
    }

    /// Get a project entry by name
    #[allow(dead_code)]
    pub fn get_project(&self, project_name: &str) -> Option<&ProjectEntry> {
        self.projects.get(project_name)
    }

    /// Get a project entry by path
    #[allow(dead_code)]
    pub fn get_project_by_path(&self, path: &PathBuf) -> Option<&ProjectEntry> {
        self.projects.values().find(|entry| entry.path == *path)
    }

    /// List all registered projects
    pub fn list_projects(&self) -> Vec<&ProjectEntry> {
        let mut projects: Vec<&ProjectEntry> = self.projects.values().collect();
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        projects
    }

    /// Check if a project is registered by name
    #[allow(dead_code)]
    pub fn is_registered(&self, project_name: &str) -> bool {
        self.projects.contains_key(project_name)
    }

    /// Check if a project is registered by path
    #[allow(dead_code)]
    pub fn is_registered_by_path(&self, path: &PathBuf) -> bool {
        self.projects.values().any(|entry| entry.path == *path)
    }
}

impl Default for PortRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_conflict_detection() {
        let mut registry = PortRegistry::new();

        // Register first project
        let entry1 = ProjectEntry {
            name: "project-a".to_string(),
            path: PathBuf::from("/path/to/project-a"),
            domain: "project-a.local".to_string(),
            network: "project-a-net".to_string(),
            ports: vec![5432, 6379, 8080],
            containers: vec!["project-a-postgres".to_string()],
        };
        registry.projects.insert(entry1.name.clone(), entry1);

        // Check for conflicts with overlapping ports
        let conflicts = registry.check_port_conflicts("project-b", &[5432, 3000]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, 5432);
        assert_eq!(conflicts[0].1, "project-a");

        // Check for conflicts with no overlapping ports
        let conflicts = registry.check_port_conflicts("project-b", &[3000, 3001]);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_project_registration() {
        let mut registry = PortRegistry::new();

        let entry = ProjectEntry {
            name: "test-project".to_string(),
            path: PathBuf::from("/path/to/test"),
            domain: "test.local".to_string(),
            network: "test-net".to_string(),
            ports: vec![5432],
            containers: vec!["test-postgres".to_string()],
        };

        registry.projects.insert(entry.name.clone(), entry);

        assert!(registry.is_registered("test-project"));
        assert!(registry.is_registered_by_path(&PathBuf::from("/path/to/test")));

        let retrieved = registry.get_project("test-project").unwrap();
        assert_eq!(retrieved.name, "test-project");
        assert_eq!(retrieved.ports.len(), 1);
    }
}
