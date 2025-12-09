//! /etc/hosts file management
//!
//! This module handles adding and removing domain entries from /etc/hosts
//! for local development domains with backup support.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;

use crate::config::get_config_dir;

/// Marker for oh-my-dockers managed entries in /etc/hosts
const MARKER_PREFIX: &str = "# oh-my-dockers";
/// Separator for marking oh-my-dockers managed entries
const SECTION_START_PREFIX: &str = "# === oh-my-dockers start ===";
const SECTION_END_PREFIX: &str = "# === oh-my-dockers end ===";

/// Represents a project's hosts entries
#[derive(Debug, Clone)]
struct ProjectSection {
    project_name: String,
    domains: Vec<String>,
    start_line: usize,
    end_line: usize,
}

/// Parse all oh-my-dockers managed sections from hosts file
fn parse_hosts_file(content: &str) -> (Vec<String>, HashMap<String, ProjectSection>) {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut sections: HashMap<String, ProjectSection> = HashMap::new();

    let mut current_project: Option<String> = None;
    let mut current_start: Option<usize> = None;
    let mut current_domains: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for section start
        if trimmed.starts_with(SECTION_START_PREFIX) {
            // Extract project name
            let project_name = trimmed
                .strip_prefix(SECTION_START_PREFIX)
                .unwrap_or("")
                .trim()
                .to_string();

            if !project_name.is_empty() {
                current_project = Some(project_name);
                current_start = Some(idx);
                current_domains.clear();
            }
        }
        // Check for section end
        else if trimmed.starts_with(SECTION_END_PREFIX) {
            if let (Some(project), Some(start)) = (&current_project, current_start) {
                let project_name = trimmed
                    .strip_prefix(SECTION_END_PREFIX)
                    .unwrap_or("")
                    .trim()
                    .to_string();

                // Only close if it matches the current project
                if project_name.is_empty() || &project_name == project {
                    sections.insert(
                        project.clone(),
                        ProjectSection {
                            project_name: project.clone(),
                            domains: current_domains.clone(),
                            start_line: start,
                            end_line: idx + 1, // Include the end marker
                        },
                    );
                    current_project = None;
                    current_start = None;
                    current_domains.clear();
                }
            }
        }
        // Collect domains within a section
        else if current_project.is_some() && trimmed.starts_with("127.0.0.1") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                current_domains.push(parts[1].to_string());
            }
        }
    }

    // Handle unclosed sections (legacy format or corrupted)
    if let (Some(project), Some(start)) = (current_project, current_start) {
        sections.insert(
            project.clone(),
            ProjectSection {
                project_name: project,
                domains: current_domains,
                start_line: start,
                end_line: lines.len(),
            },
        );
    }

    (lines, sections)
}

/// Create a backup of the hosts file
fn backup_hosts_file() -> Result<PathBuf> {
    let hosts_path = Path::new("/etc/hosts");
    let config_dir = get_config_dir()?;
    let backup_dir = config_dir.join("backups").join("hosts");

    // Create backup directory if it doesn't exist
    fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;

    // Generate backup filename with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("hosts_{}.bak", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    // Read and copy hosts file
    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;
    fs::write(&backup_path, &content).context("Failed to write backup file")?;

    // Keep only last 10 backups
    cleanup_old_backups(&backup_dir, 10)?;

    Ok(backup_path)
}

/// Remove old backups, keeping only the most recent `keep_count`
fn cleanup_old_backups(backup_dir: &Path, keep_count: usize) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "bak")
                .unwrap_or(false)
        })
        .collect();

    // Sort by modification time (newest first)
    entries.sort_by(|a, b| {
        b.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .cmp(
                &a.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            )
    });

    // Remove old backups
    for entry in entries.into_iter().skip(keep_count) {
        let _ = fs::remove_file(entry.path());
    }

    Ok(())
}

/// Find all existing domains in the hosts file (not managed by oh-my-dockers)
fn find_unmanaged_domains(lines: &[String], sections: &HashMap<String, ProjectSection>) -> HashSet<String> {
    let mut unmanaged: HashSet<String> = HashSet::new();
    
    // Collect all managed line ranges
    let managed_ranges: Vec<(usize, usize)> = sections
        .values()
        .map(|s| (s.start_line, s.end_line))
        .collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip if this line is in a managed section
        let in_managed = managed_ranges.iter().any(|(start, end)| idx >= *start && idx < *end);
        if in_managed {
            continue;
        }

        // Check if this is a 127.0.0.1 entry
        if trimmed.starts_with("127.0.0.1") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                unmanaged.insert(parts[1].to_string());
            }
        }
    }

    unmanaged
}

/// Build the new hosts file content
fn build_hosts_content(
    original_lines: &[String],
    sections: &HashMap<String, ProjectSection>,
    project_name: &str,
    new_domains: Option<&[String]>,
) -> String {
    let mut result_lines: Vec<String> = Vec::new();
    let mut skip_until: Option<usize> = None;

    // Collect all sections sorted by start line
    let mut sorted_sections: Vec<&ProjectSection> = sections.values().collect();
    sorted_sections.sort_by_key(|s| s.start_line);

    for (idx, line) in original_lines.iter().enumerate() {
        // Skip lines that are part of sections we're removing/replacing
        if let Some(skip_end) = skip_until {
            if idx < skip_end {
                continue;
            }
            skip_until = None;
        }

        // Check if this line starts a section we need to handle
        let mut handled = false;
        for section in &sorted_sections {
            if idx == section.start_line {
                if section.project_name == project_name {
                    // Skip this section (we'll add the new one at the end)
                    skip_until = Some(section.end_line);
                    handled = true;
                    break;
                }
            }
        }

        if !handled {
            result_lines.push(line.clone());
        }
    }

    // Remove trailing empty lines
    while result_lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        result_lines.pop();
    }

    // Add new section if domains are provided
    if let Some(domains) = new_domains {
        if !domains.is_empty() {
            result_lines.push(String::new()); // Empty line before section
            result_lines.push(format!("{} {}", SECTION_START_PREFIX, project_name));
            result_lines.push(format!("{} {}", MARKER_PREFIX, project_name));
            for domain in domains {
                result_lines.push(format!("127.0.0.1 {}", domain));
            }
            result_lines.push(format!("{} {}", SECTION_END_PREFIX, project_name));
        }
    }

    // Ensure file ends with newline
    result_lines.join("\n") + "\n"
}

/// Add domains to /etc/hosts for a project
pub fn add_project_domains(project_name: &str, domains: &[String]) -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    // Read and parse current hosts file
    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;
    let (lines, sections) = parse_hosts_file(&content);

    // Find unmanaged domains to check for conflicts
    let unmanaged_domains = find_unmanaged_domains(&lines, &sections);

    // Deduplicate and filter domains
    let mut domains_to_add: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut skipped_domains: Vec<(String, String)> = Vec::new(); // (domain, reason)

    for domain in domains {
        // Skip duplicates in input
        if seen.contains(domain) {
            continue;
        }
        seen.insert(domain.clone());

        // Check if already managed by another project
        let mut already_managed = false;
        for (other_project, section) in &sections {
            if other_project != project_name && section.domains.contains(domain) {
                skipped_domains.push((
                    domain.clone(),
                    format!("already managed by project '{}'", other_project),
                ));
                already_managed = true;
                break;
            }
        }

        if already_managed {
            continue;
        }

        // Check if exists as unmanaged entry
        if unmanaged_domains.contains(domain) {
            skipped_domains.push((
                domain.clone(),
                "exists as unmanaged entry in /etc/hosts".to_string(),
            ));
            continue;
        }

        domains_to_add.push(domain.clone());
    }

    // Check if the hosts file already has the exact same entries for this project
    if let Some(existing) = sections.get(project_name) {
        let existing_set: HashSet<&String> = existing.domains.iter().collect();
        let new_set: HashSet<&String> = domains_to_add.iter().collect();

        if existing_set == new_set {
            // No changes needed
            println!(
                "{} /etc/hosts already up to date for project {}",
                "✓".green(),
                project_name.bright_white()
            );
            return Ok(());
        }
    } else if domains_to_add.is_empty() {
        // No existing section and no domains to add
        println!(
            "{} No domains to add to /etc/hosts for project {}",
            "ℹ".blue(),
            project_name.bright_white()
        );
        return Ok(());
    }

    // Show existing entries for this project
    if let Some(existing) = sections.get(project_name) {
        println!();
        println!(
            "{} Found existing entries for project {}:",
            "ℹ".blue(),
            project_name.bright_white()
        );
        for domain in &existing.domains {
            println!("  127.0.0.1 {}", domain);
        }
    }

    // Show skipped domains
    if !skipped_domains.is_empty() {
        println!();
        println!("{} Skipped domains:", "⚠".yellow());
        for (domain, reason) in &skipped_domains {
            println!("  {} - {}", domain.bright_white(), reason);
        }
    }

    // Show preview
    println!();
    println!("{} Preview of changes to /etc/hosts:", "ℹ".blue());
    println!();
    if sections.contains_key(project_name) {
        println!("{}", "Will replace existing section with:".bright_white());
    } else {
        println!("{}", "Will add the following entries:".bright_white());
    }
    println!();
    println!("  {} {}", SECTION_START_PREFIX, project_name);
    println!("  {} {}", MARKER_PREFIX, project_name);
    for domain in &domains_to_add {
        println!("  127.0.0.1 {}", domain);
    }
    println!("  {} {}", SECTION_END_PREFIX, project_name);
    println!();

    // Ask for confirmation
    print!("{} Apply these changes? [Y/n]: ", "?".bright_yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("n") || trimmed.eq_ignore_ascii_case("no") {
        println!("{} Changes cancelled", "ℹ".blue());
        return Ok(());
    }

    // Create backup before modifying
    match backup_hosts_file() {
        Ok(backup_path) => {
            println!(
                "{} Backup created: {}",
                "✓".green(),
                backup_path.display()
            );
        }
        Err(e) => {
            println!(
                "{} Warning: Could not create backup: {}",
                "⚠".yellow(),
                e
            );
        }
    }

    // Build new content
    let new_content = build_hosts_content(&lines, &sections, project_name, Some(&domains_to_add));

    // Write back to file
    write_hosts_file(hosts_path, &new_content)?;

    println!(
        "{} Updated /etc/hosts with {} domain(s) for project {}",
        "✓".green(),
        domains_to_add.len(),
        project_name.bright_white()
    );

    Ok(())
}

/// Remove domains from /etc/hosts for a project
pub fn remove_project_domains(project_name: &str) -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    if !hosts_path.exists() {
        return Ok(());
    }

    // Read and parse current hosts file
    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;
    let (lines, sections) = parse_hosts_file(&content);

    // Check if project has entries
    let section = match sections.get(project_name) {
        Some(s) => s,
        None => {
            println!(
                "{} No entries found for project {}",
                "ℹ".blue(),
                project_name.bright_white()
            );
            return Ok(());
        }
    };

    // Show preview
    println!();
    println!("{} Preview of changes to /etc/hosts:", "ℹ".blue());
    println!();
    println!("{}", "Will remove the following entries:".bright_white());
    println!();
    println!("  {} {}", SECTION_START_PREFIX, project_name);
    println!("  {} {}", MARKER_PREFIX, project_name);
    for domain in &section.domains {
        println!("  127.0.0.1 {}", domain);
    }
    println!("  {} {}", SECTION_END_PREFIX, project_name);
    println!();

    // Ask for confirmation
    print!("{} Apply these changes? [Y/n]: ", "?".bright_yellow());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("n") || trimmed.eq_ignore_ascii_case("no") {
        println!("{} Changes cancelled", "ℹ".blue());
        return Ok(());
    }

    // Create backup before modifying
    match backup_hosts_file() {
        Ok(backup_path) => {
            println!(
                "{} Backup created: {}",
                "✓".green(),
                backup_path.display()
            );
        }
        Err(e) => {
            println!(
                "{} Warning: Could not create backup: {}",
                "⚠".yellow(),
                e
            );
        }
    }

    // Build new content without this project's section
    let new_content = build_hosts_content(&lines, &sections, project_name, None);

    // Write back to file
    write_hosts_file(hosts_path, &new_content)?;

    println!(
        "{} Removed {} domain(s) for project {} from /etc/hosts",
        "✓".green(),
        section.domains.len(),
        project_name.bright_white()
    );

    Ok(())
}

/// List all domains managed by oh-my-dockers
pub fn list_managed_domains() -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    if !hosts_path.exists() {
        println!("{}", "No /etc/hosts file found".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;
    let (_, sections) = parse_hosts_file(&content);

    if sections.is_empty() {
        println!("{}", "No oh-my-dockers managed domains found".yellow());
        return Ok(());
    }

    println!("{}", "oh-my-dockers managed domains:".blue());
    println!();

    // Sort projects alphabetically
    let mut projects: Vec<_> = sections.keys().collect();
    projects.sort();

    for project in projects {
        if let Some(section) = sections.get(project) {
            println!("  {} {}", "•".bright_white(), project.bright_white());
            for domain in &section.domains {
                println!("    - {}", domain);
            }
            println!();
        }
    }

    Ok(())
}

/// Clean up all oh-my-dockers managed entries from /etc/hosts
pub fn cleanup_all_domains() -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    if !hosts_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;
    let (lines, sections) = parse_hosts_file(&content);

    if sections.is_empty() {
        println!("{}", "No oh-my-dockers managed domains found".yellow());
        return Ok(());
    }

    // Show preview
    println!();
    println!("{} Preview of changes to /etc/hosts:", "ℹ".blue());
    println!();
    println!(
        "{}",
        "Will remove ALL oh-my-dockers managed entries:".bright_white()
    );
    println!();

    let mut total_domains = 0;
    for (project, section) in &sections {
        println!("  Project: {}", project.bright_white());
        for domain in &section.domains {
            println!("    - {}", domain);
            total_domains += 1;
        }
    }
    println!();
    println!(
        "  Total: {} project(s), {} domain(s)",
        sections.len(),
        total_domains
    );
    println!();

    // Ask for confirmation
    print!(
        "{} Are you sure you want to remove ALL entries? [y/N]: ",
        "?".bright_yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
        println!("{} Changes cancelled", "ℹ".blue());
        return Ok(());
    }

    // Create backup before modifying
    match backup_hosts_file() {
        Ok(backup_path) => {
            println!(
                "{} Backup created: {}",
                "✓".green(),
                backup_path.display()
            );
        }
        Err(e) => {
            println!(
                "{} Warning: Could not create backup: {}",
                "⚠".yellow(),
                e
            );
        }
    }

    // Build content with all sections removed
    let mut result_lines: Vec<String> = Vec::new();
    let mut skip_ranges: Vec<(usize, usize)> = sections
        .values()
        .map(|s| (s.start_line, s.end_line))
        .collect();
    skip_ranges.sort_by_key(|r| r.0);

    let mut skip_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        // Check if we should skip this line
        while skip_idx < skip_ranges.len() && idx >= skip_ranges[skip_idx].1 {
            skip_idx += 1;
        }

        if skip_idx < skip_ranges.len()
            && idx >= skip_ranges[skip_idx].0
            && idx < skip_ranges[skip_idx].1
        {
            continue;
        }

        result_lines.push(line.clone());
    }

    // Remove trailing empty lines
    while result_lines
        .last()
        .map(|l| l.trim().is_empty())
        .unwrap_or(false)
    {
        result_lines.pop();
    }

    let new_content = result_lines.join("\n") + "\n";

    // Write back to file
    write_hosts_file(hosts_path, &new_content)?;

    println!(
        "{} Removed all oh-my-dockers managed entries ({} project(s), {} domain(s))",
        "✓".green(),
        sections.len(),
        total_domains
    );

    Ok(())
}

/// Write content to the hosts file, using sudo if necessary
fn write_hosts_file(hosts_path: &Path, content: &str) -> Result<()> {
    // Try to write directly first
    if let Err(e) = fs::write(hosts_path, content) {
        // If direct write fails, try using sudo tee
        println!("{} Attempting to write with sudo privileges...", "ℹ".blue());

        // Use sudo tee to write the file
        let mut child = Command::new("sudo")
            .arg("tee")
            .arg(hosts_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to execute sudo tee. Make sure sudo is available.")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .context("Failed to write to sudo tee stdin")?;
            // Close stdin to signal EOF
            drop(stdin);
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for sudo tee")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Failed to write /etc/hosts with sudo: {}. Error: {}. Please run with sudo or manually add entries.",
                e,
                error_msg
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hosts_file() {
        let content = r#"127.0.0.1 localhost
::1 localhost

# === oh-my-dockers start === project-a
# oh-my-dockers project-a
127.0.0.1 project-a.local
127.0.0.1 api.project-a.local
# === oh-my-dockers end === project-a

# === oh-my-dockers start === project-b
# oh-my-dockers project-b
127.0.0.1 project-b.local
# === oh-my-dockers end === project-b
"#;

        let (lines, sections) = parse_hosts_file(content);

        assert_eq!(lines.len(), 13);
        assert_eq!(sections.len(), 2);

        let section_a = sections.get("project-a").unwrap();
        assert_eq!(section_a.domains.len(), 2);
        assert!(section_a.domains.contains(&"project-a.local".to_string()));
        assert!(section_a.domains.contains(&"api.project-a.local".to_string()));

        let section_b = sections.get("project-b").unwrap();
        assert_eq!(section_b.domains.len(), 1);
        assert!(section_b.domains.contains(&"project-b.local".to_string()));
    }

    #[test]
    fn test_build_hosts_content_add() {
        let content = "127.0.0.1 localhost\n::1 localhost\n";
        let (lines, sections) = parse_hosts_file(content);

        let new_domains = vec!["test.local".to_string(), "api.test.local".to_string()];
        let result = build_hosts_content(&lines, &sections, "test-project", Some(&new_domains));

        assert!(result.contains("# === oh-my-dockers start === test-project"));
        assert!(result.contains("127.0.0.1 test.local"));
        assert!(result.contains("127.0.0.1 api.test.local"));
        assert!(result.contains("# === oh-my-dockers end === test-project"));
    }

    #[test]
    fn test_build_hosts_content_remove() {
        let content = r#"127.0.0.1 localhost

# === oh-my-dockers start === test-project
# oh-my-dockers test-project
127.0.0.1 test.local
# === oh-my-dockers end === test-project
"#;
        let (lines, sections) = parse_hosts_file(content);

        let result = build_hosts_content(&lines, &sections, "test-project", None);

        assert!(!result.contains("test-project"));
        assert!(!result.contains("test.local"));
        assert!(result.contains("127.0.0.1 localhost"));
    }
}
