//! CLI command definitions for oh-my-dockers
//!
//! This module contains all the clap-based command definitions and argument parsing.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "omd")]
#[command(about = "Manage Docker development environments", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize omd.toml in current directory
    Init,
    /// Manage Caddy reverse proxy
    Caddy {
        #[command(subcommand)]
        subcommand: CaddyCommands,
    },
    /// Manage Docker networks
    Network {
        #[command(subcommand)]
        subcommand: NetworkCommands,
    },
    /// Manage reverse proxy configurations
    Proxy {
        #[command(subcommand)]
        subcommand: ProxyCommands,
    },
    /// Display port mappings
    Ports {
        /// Show port mappings for a specific network
        #[arg(value_name = "NETWORK")]
        network: Option<String>,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        subcommand: ProjectCommands,
    },
    /// Manage /etc/hosts entries
    Hosts {
        #[command(subcommand)]
        subcommand: HostsCommands,
    },
}

#[derive(Subcommand)]
pub enum CaddyCommands {
    /// Start Caddy container
    Start,
    /// Stop Caddy container
    Stop,
    /// Restart Caddy container
    Restart,
    /// Show Caddy status
    Status,
    /// Show Caddy logs
    Logs {
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// List all networks
    List,
}

#[derive(Subcommand)]
pub enum ProxyCommands {
    /// Add a reverse proxy rule
    Add { domain: String, target: String },
    /// Remove a reverse proxy rule
    Remove { domain: String },
    /// List all proxy rules
    List,
    /// Reload Caddy configuration
    Reload,
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List all registered projects
    List,
    /// Configure project and start containers (run from project directory)
    Up,
    /// Stop containers (run from project directory)
    Down,
    /// Stop containers and remove all project configuration (run from project directory)
    Remove,
}

#[derive(Subcommand)]
pub enum HostsCommands {
    /// List all domains managed by oh-my-dockers
    List,
    /// Remove all oh-my-dockers managed entries from /etc/hosts
    Cleanup,
}
