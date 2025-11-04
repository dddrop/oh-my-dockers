use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod network;
mod ports;
mod project;
mod proxy;

#[derive(Parser)]
#[command(name = "oh-my-dockers")]
#[command(about = "Manage Docker development environments", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        network: Option<String>,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        subcommand: ProjectCommands,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Create a new network
    Create { name: String },
    /// List all networks
    List,
    /// Remove a network
    Remove { name: String },
    /// Connect a container to a network
    Connect {
        network: String,
        container: String,
    },
}

#[derive(Subcommand)]
enum ProxyCommands {
    /// Add a reverse proxy rule
    Add {
        domain: String,
        target: String,
    },
    /// Remove a reverse proxy rule
    Remove { domain: String },
    /// List all proxy rules
    List,
    /// Reload Caddy configuration
    Reload,
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// List all projects
    List,
    /// Start a project
    Up { project: String },
    /// Stop a project
    Down { project: String },
}

fn main() -> Result<()> {
    // Ensure configuration directory exists on startup
    config::ensure_config_dir()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Network { subcommand } => match subcommand {
            NetworkCommands::Create { name } => {
                network::create(&name)?;
            }
            NetworkCommands::List => {
                network::list()?;
            }
            NetworkCommands::Remove { name } => {
                network::remove(&name)?;
            }
            NetworkCommands::Connect { network, container } => {
                network::connect(&network, &container)?;
            }
        },
        Commands::Proxy { subcommand } => match subcommand {
            ProxyCommands::Add { domain, target } => {
                proxy::add(&domain, &target)?;
            }
            ProxyCommands::Remove { domain } => {
                proxy::remove(&domain)?;
            }
            ProxyCommands::List => {
                proxy::list()?;
            }
            ProxyCommands::Reload => {
                proxy::reload()?;
            }
        },
        Commands::Ports { network } => {
            if let Some(net) = network {
                ports::show(&net)?;
            } else {
                ports::list()?;
            }
        }
        Commands::Project { subcommand } => match subcommand {
            ProjectCommands::List => {
                project::list()?;
            }
            ProjectCommands::Up { project } => {
                project::up(&project)?;
            }
            ProjectCommands::Down { project } => {
                project::down(&project)?;
            }
        },
    }

    Ok(())
}

