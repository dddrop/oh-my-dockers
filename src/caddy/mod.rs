//! Caddy reverse proxy management module
//!
//! This module contains all functionality related to the Caddy reverse proxy:
//! - Container lifecycle management (start, stop, restart, status)
//! - Project-specific Caddy configuration generation
//! - Manual proxy rule management

pub mod config;
pub mod manager;
pub mod proxy;

/// The name of the Caddy container managed by oh-my-dockers
pub const CADDY_CONTAINER_NAME: &str = "oh-my-dockers-caddy";

/// The default Caddy network name
pub const CADDY_NETWORK_NAME: &str = "caddy-net";

/// The Docker label used to identify oh-my-dockers managed services
pub const OMD_SERVICE_LABEL: &str = "com.oh-my-dockers.service";
