use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

mod diff;
mod generator;
mod parser;
mod writer;

use diff::show_diff;
use generator::generate_entry;
use parser::HostsFile;
use writer::write_hosts_file;

#[derive(Parser)]
#[command(name = "hosts")]
#[command(about = "Manage /etc/hosts entries for oh-my-dockers projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add project domains to /etc/hosts
    Add {
        /// Project name
        project: String,
    },
    /// Remove project domains from /etc/hosts
    Remove {
        /// Project name
        project: String,
    },
    /// List all oh-my-dockers managed hosts entries
    List,
    /// Clean all oh-my-dockers managed hosts entries
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { project } => add_project(&project)?,
        Commands::Remove { project } => remove_project(&project)?,
        Commands::List => list_entries()?,
        Commands::Clean => clean_all()?,
    }

    Ok(())
}

fn add_project(project: &str) -> Result<()> {
    println!("{} /etc/hosts...", "Reading".blue());

    // Load current hosts file
    let mut hosts = HostsFile::load()?;

    // Generate new entry
    let entry = generate_entry(project)?;

    // Check if already exists
    if hosts.has_project(project) {
        println!(
            "{} Project {} already in /etc/hosts",
            "✓".green(),
            project.bright_white()
        );
        return Ok(());
    }

    // Add entry
    hosts.add_entry(project, &entry);

    // Show diff
    let current_content =
        std::fs::read_to_string("/etc/hosts").context("Failed to read /etc/hosts")?;
    let new_content = hosts.to_string();

    println!();
    println!("{}", "Changes to be applied:".bright_white());
    show_diff(&current_content, &new_content);

    // Confirm
    if !confirm_changes()? {
        println!("{}", "Cancelled".yellow());
        return Ok(());
    }

    // Write
    write_hosts_file(&new_content)?;

    println!();
    println!("{} Hosts file updated successfully", "✓".green());
    println!();
    println!("Added domains:");

    // Extract all domains from the entry
    if let Some(line) = entry.lines().find(|l| l.contains("127.0.0.1")) {
        let domains: Vec<&str> = line.split_whitespace().skip(1).collect();
        for domain in domains {
            println!("  {} {}", "•".bright_white(), domain);
        }
    }

    Ok(())
}

fn remove_project(project: &str) -> Result<()> {
    println!("{} /etc/hosts...", "Reading".blue());

    // Load current hosts file
    let mut hosts = HostsFile::load()?;

    // Check if exists
    if !hosts.has_project(project) {
        println!(
            "{} Project {} not found in /etc/hosts",
            "⚠".yellow(),
            project.bright_white()
        );
        return Ok(());
    }

    // Remove entry
    hosts.remove_entry(project);

    // Show diff
    let current_content =
        std::fs::read_to_string("/etc/hosts").context("Failed to read /etc/hosts")?;
    let new_content = hosts.to_string();

    println!();
    println!("{}", "Changes to be applied:".bright_white());
    show_diff(&current_content, &new_content);

    // Confirm
    if !confirm_changes()? {
        println!("{}", "Cancelled".yellow());
        return Ok(());
    }

    // Write
    write_hosts_file(&new_content)?;

    println!();
    println!("{} Hosts file updated successfully", "✓".green());
    println!();
    println!("Removed project: {}", project.bright_white());

    Ok(())
}

fn list_entries() -> Result<()> {
    let hosts = HostsFile::load()?;
    let projects = hosts.list_managed_projects();

    if projects.is_empty() {
        println!(
            "{}",
            "No oh-my-dockers entries found in /etc/hosts".yellow()
        );
    } else {
        println!("{}", "oh-my-dockers managed entries:".blue());
        println!();
        for project in projects {
            println!("  {} {}", "•".bright_white(), project.bright_white());
        }
    }

    Ok(())
}

fn clean_all() -> Result<()> {
    println!("{} /etc/hosts...", "Reading".blue());

    let mut hosts = HostsFile::load()?;
    let projects = hosts.list_managed_projects();

    if projects.is_empty() {
        println!("{}", "No oh-my-dockers entries to clean".yellow());
        return Ok(());
    }

    // Remove all managed entries
    for project in &projects {
        hosts.remove_entry(project);
    }

    // Show diff
    let current_content =
        std::fs::read_to_string("/etc/hosts").context("Failed to read /etc/hosts")?;
    let new_content = hosts.to_string();

    println!();
    println!("{}", "Changes to be applied:".bright_white());
    show_diff(&current_content, &new_content);

    // Confirm
    if !confirm_changes()? {
        println!("{}", "Cancelled".yellow());
        return Ok(());
    }

    // Write
    write_hosts_file(&new_content)?;

    println!();
    println!(
        "{} Cleaned {} project(s) from /etc/hosts",
        "✓".green(),
        projects.len()
    );

    Ok(())
}

fn confirm_changes() -> Result<bool> {
    use std::io::{self, Write};

    println!();
    print!("Apply these changes to /etc/hosts? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}
