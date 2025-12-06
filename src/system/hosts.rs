//! /etc/hosts file management
//!
//! This module handles adding and removing domain entries from /etc/hosts
//! for local development domains.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

/// Marker for oh-my-dockers managed entries in /etc/hosts
const MARKER: &str = "# oh-my-dockers";
/// Separator for marking oh-my-dockers managed entries
const SEPARATOR_START: &str = "# === oh-my-dockers start ===";
const SEPARATOR_END: &str = "# === oh-my-dockers end ===";

/// Add domains to /etc/hosts for a project
pub fn add_project_domains(project_name: &str, domains: &[String]) -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    // Read current hosts file (reading usually doesn't require sudo)
    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;

    // Parse existing entries
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find existing entries for this project to remove
    let marker = format!("{} {}", MARKER, project_name);
    let separator_start = format!("{} {}", SEPARATOR_START, project_name);
    let separator_end = format!("{} {}", SEPARATOR_END, project_name);

    let (project_start_idx, project_end_idx) =
        find_project_section(&lines, &marker, &separator_start, &separator_end);

    // Show preview of what will be removed (if exists)
    if let (Some(start), Some(end)) = (project_start_idx, project_end_idx) {
        println!();
        println!(
            "{} Found existing entries for project {}:",
            "ℹ".blue(),
            project_name.bright_white()
        );
        println!();
        for idx in start..end.min(lines.len()) {
            println!("  {}", lines[idx]);
        }
        println!();
    }

    // Build new entries with separators
    let mut new_entries = Vec::new();
    new_entries.push(format!("{} {}", SEPARATOR_START, project_name));
    new_entries.push(format!("{} {}", MARKER, project_name));

    let mut domains_to_add = Vec::new();
    for domain in domains {
        // Check if domain already exists (not managed by us)
        let domain_entry = format!("127.0.0.1 {}", domain);
        if !lines.iter().any(|line| {
            let trimmed = line.trim();
            trimmed == domain_entry || trimmed.starts_with(&format!("127.0.0.1 {} ", domain))
        }) {
            domains_to_add.push(domain.clone());
            new_entries.push(format!("127.0.0.1 {}", domain));
        } else {
            println!(
                "{} Domain {} already exists in /etc/hosts (not managed by oh-my-dockers)",
                "⚠".yellow(),
                domain.bright_white()
            );
        }
    }

    new_entries.push(format!("{} {}", SEPARATOR_END, project_name));

    // Show preview
    println!();
    println!("{} Preview of changes to /etc/hosts:", "ℹ".blue());
    println!();
    if project_start_idx.is_some() || project_end_idx.is_some() {
        println!(
            "{}",
            "Will replace the existing entries with:".bright_white()
        );
    } else {
        println!("{}", "Will add the following entries:".bright_white());
    }
    println!();
    for entry in &new_entries {
        println!("  {}", entry);
    }
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

    // Remove existing entries for this project (if any)
    if let (Some(start), Some(end)) = (project_start_idx, project_end_idx) {
        // Handle case where end might be beyond lines.len()
        let end_idx = end.min(lines.len());
        if start < end_idx {
            lines.drain(start..end_idx);
        }
    }

    // Add new entries to lines
    lines.extend(new_entries);

    // Write back to file
    let new_content = lines.join("\n") + "\n";

    write_hosts_file(hosts_path, &new_content)?;

    println!(
        "{} Added {} domain(s) to /etc/hosts",
        "✓".green(),
        domains_to_add.len()
    );

    Ok(())
}

/// Remove domains from /etc/hosts for a project
pub fn remove_project_domains(project_name: &str) -> Result<()> {
    let hosts_path = Path::new("/etc/hosts");

    if !hosts_path.exists() {
        return Ok(());
    }

    // Read current hosts file (reading usually doesn't require sudo)
    let content = fs::read_to_string(hosts_path).context("Failed to read /etc/hosts")?;

    // Find and remove entries for this project
    let marker = format!("{} {}", MARKER, project_name);
    let separator_start = format!("{} {}", SEPARATOR_START, project_name);
    let separator_end = format!("{} {}", SEPARATOR_END, project_name);

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let (project_start_idx, project_end_idx) =
        find_project_section(&lines, &marker, &separator_start, &separator_end);

    if let (Some(start), Some(end)) = (project_start_idx, project_end_idx) {
        // Show preview
        println!();
        println!("{} Preview of changes to /etc/hosts:", "ℹ".blue());
        println!();
        println!("{}", "Will remove the following entries:".bright_white());
        println!();
        let end_idx = end.min(lines.len());
        for idx in start..end_idx {
            println!("  {}", lines[idx]);
        }
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

        // Remove the entries
        let end_idx = end.min(lines.len());
        if start < end_idx {
            lines.drain(start..end_idx);
        }
    } else {
        println!(
            "{} No entries found for project {}",
            "ℹ".blue(),
            project_name.bright_white()
        );
        return Ok(());
    }

    // Write back to file
    let new_content = lines.join("\n");
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        new_content + "\n"
    };

    write_hosts_file(hosts_path, &new_content)?;

    println!("{} Removed project domains from /etc/hosts", "✓".green());

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

    let mut project_domains: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_project: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this is a project marker
        if trimmed.starts_with(MARKER) {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                current_project = Some(parts[2].to_string());
            }
        } else if let Some(project) = &current_project {
            // Check if this is a domain entry
            if trimmed.starts_with("127.0.0.1") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let domain = parts[1].to_string();
                    project_domains
                        .entry(project.clone())
                        .or_default()
                        .push(domain);
                }
            }
        }
    }

    if project_domains.is_empty() {
        println!("{}", "No oh-my-dockers managed domains found".yellow());
        return Ok(());
    }

    println!("{}", "oh-my-dockers managed domains:".blue());
    println!();

    for (project, domains) in project_domains {
        println!("  {} {}", "•".bright_white(), project.bright_white());
        for domain in domains {
            println!("    - {}", domain);
        }
        println!();
    }

    Ok(())
}

/// Find the start and end indices of a project's section in the hosts file
fn find_project_section(
    lines: &[String],
    marker: &str,
    separator_start: &str,
    separator_end: &str,
) -> (Option<usize>, Option<usize>) {
    let mut project_start_idx = None;
    let mut project_end_idx = None;
    let mut in_project_section = false;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check if this is the start of our project section
        if trimmed == separator_start || trimmed == marker {
            if project_start_idx.is_none() {
                project_start_idx = Some(idx);
            }
            in_project_section = true;
        }
        // Check if this is the end of our project section
        else if in_project_section && trimmed == separator_end {
            project_end_idx = Some(idx + 1); // Include the end separator
            break;
        }
        // If we're in the section, check if we've hit the next section or non-domain line
        else if in_project_section && project_end_idx.is_none() {
            // If we hit another project's separator start, that's our end
            if trimmed.starts_with(SEPARATOR_START) && trimmed != separator_start {
                project_end_idx = Some(idx);
                break;
            }
            // If we hit a non-domain, non-empty, non-comment line, that might be our end
            else if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("127.0.0.1")
            {
                // But first check if it's a valid hosts entry (could be IPv6)
                if !trimmed.contains("::") && !trimmed.contains('\t') && !trimmed.contains(' ') {
                    project_end_idx = Some(idx);
                    break;
                }
            }
        }
    }

    // If we found start but no end, set end to end of file
    if project_start_idx.is_some() && project_end_idx.is_none() {
        project_end_idx = Some(lines.len());
    }

    (project_start_idx, project_end_idx)
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
