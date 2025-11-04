use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod docker_compose;
mod init;
mod network;
mod ports;
mod project;
mod proxy;
mod registry;

#[derive(Parser)]
#[command(name = "omd")]
#[command(about = "Manage Docker development environments", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize omd.toml in current directory
    Init,
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
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// List all networks
    List,
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
    /// List all registered projects
    List,
    /// Configure project (run from project directory)
    Up,
    /// Remove project configuration (run from project directory)
    Down,
}

fn main() -> Result<()> {
    // Ensure configuration directory exists on startup
    config::ensure_config_dir()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init::init()?;
        }
        Commands::Network { subcommand } => match subcommand {
            NetworkCommands::List => {
                network::list()?;
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
            ProjectCommands::Up => {
                project::up()?;
            }
            ProjectCommands::Down => {
                project::down()?;
            }
        },
    }

    Ok(())
}

