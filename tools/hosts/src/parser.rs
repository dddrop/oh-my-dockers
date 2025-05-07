use std::{collections::HashMap, fs};

use anyhow::{Context, Result};

const HOSTS_PATH: &str = "/etc/hosts";
const MARKER_START: &str = "# oh-my-dockers:";
const MARKER_END: &str = "# oh-my-dockers: end";

#[derive(Debug)]
pub struct HostsFile {
    lines: Vec<String>,
    managed_blocks: HashMap<String, (usize, usize)>, // project -> (start_line, end_line)
}

impl HostsFile {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(HOSTS_PATH)
            .context("Failed to read /etc/hosts. Make sure the file exists and is readable.")?;

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let managed_blocks = Self::find_managed_blocks(&lines);

        Ok(Self {
            lines,
            managed_blocks,
        })
    }

    fn find_managed_blocks(lines: &[String]) -> HashMap<String, (usize, usize)> {
        let mut blocks = HashMap::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.starts_with(MARKER_START) {
                // Extract project name
                if let Some(project) = line.strip_prefix(MARKER_START).map(|s| s.trim()) {
                    let start = i;

                    // Find end marker
                    let mut end = start;
                    for j in (i + 1)..lines.len() {
                        if lines[j].trim() == MARKER_END {
                            end = j;
                            break;
                        }
                    }

                    blocks.insert(project.to_string(), (start, end));
                    i = end + 1;
                    continue;
                }
            }

            i += 1;
        }

        blocks
    }

    pub fn has_project(&self, project: &str) -> bool {
        self.managed_blocks.contains_key(project)
    }

    pub fn add_entry(&mut self, project: &str, entry: &str) {
        // Remove existing entry if present
        self.remove_entry(project);

        // Add new entry at the end
        if !self.lines.is_empty() && !self.lines.last().unwrap().is_empty() {
            self.lines.push(String::new());
        }

        for line in entry.lines() {
            self.lines.push(line.to_string());
        }

        // Rebuild managed blocks index
        self.managed_blocks = Self::find_managed_blocks(&self.lines);
    }

    pub fn remove_entry(&mut self, project: &str) {
        if let Some((start, end)) = self.managed_blocks.get(project).copied() {
            // Remove lines from start to end (inclusive)
            self.lines.drain(start..=end);

            // Remove empty lines before the block if any
            if start > 0
                && start <= self.lines.len()
                && self
                    .lines
                    .get(start - 1)
                    .map(|l| l.is_empty())
                    .unwrap_or(false)
            {
                self.lines.remove(start - 1);
            }

            // Rebuild managed blocks index
            self.managed_blocks = Self::find_managed_blocks(&self.lines);
        }
    }

    pub fn list_managed_projects(&self) -> Vec<String> {
        let mut projects: Vec<_> = self.managed_blocks.keys().cloned().collect();
        projects.sort();
        projects
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_managed_blocks() {
        let content = vec![
            "127.0.0.1 localhost".to_string(),
            "".to_string(),
            "# oh-my-dockers: daily".to_string(),
            "127.0.0.1 daily.local *.daily.local".to_string(),
            "# oh-my-dockers: end".to_string(),
            "".to_string(),
            "192.168.1.1 example.com".to_string(),
        ];

        let blocks = HostsFile::find_managed_blocks(&content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.get("daily"), Some(&(2, 4)));
    }
}
