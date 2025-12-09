//! oh-my-dockers (omd) - Docker development environment management CLI
//!
//! A powerful CLI tool for managing Docker development environments with
//! automatic reverse proxy configuration, network management, and port
//! conflict detection.

use anyhow::Result;
use clap::Parser;

mod caddy;
mod cli;
mod config;
mod docker;
mod ports;
mod project;
mod system;

use cli::{
    CaddyCommands, Cli, Commands, HostsCommands, NetworkCommands, ProjectCommands, ProxyCommands,
};

fn main() -> Result<()> {
    // Ensure configuration directory exists on startup
    config::ensure_config_dir()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            project::init::init()?;
        }
        Commands::Caddy { subcommand } => match subcommand {
            CaddyCommands::Start => {
                caddy::manager::start()?;
            }
            CaddyCommands::Stop => {
                caddy::manager::stop()?;
            }
            CaddyCommands::Restart => {
                caddy::manager::restart()?;
            }
            CaddyCommands::Status => {
                caddy::manager::status()?;
            }
            CaddyCommands::Logs { follow } => {
                caddy::manager::logs(follow)?;
            }
        },
        Commands::Network { subcommand } => match subcommand {
            NetworkCommands::List => {
                docker::network::list()?;
            }
        },
        Commands::Proxy { subcommand } => match subcommand {
            ProxyCommands::Add { domain, target } => {
                caddy::proxy::add(&domain, &target)?;
            }
            ProxyCommands::Remove { domain } => {
                caddy::proxy::remove(&domain)?;
            }
            ProxyCommands::List => {
                caddy::proxy::list()?;
            }
            ProxyCommands::Reload => {
                caddy::proxy::reload()?;
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
                project::commands::list()?;
            }
            ProjectCommands::Up => {
                project::commands::up()?;
            }
            ProjectCommands::Down => {
                project::commands::down()?;
            }
            ProjectCommands::Remove => {
                project::commands::remove()?;
            }
        },
        Commands::Hosts { subcommand } => match subcommand {
            HostsCommands::List => {
                system::hosts::list_managed_domains()?;
            }
        },
    }

    Ok(())
}
