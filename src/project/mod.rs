//! Project management module
//!
//! This module contains functionality for managing projects:
//! - Project configuration (omd.toml)
//! - Project registry
//! - Project initialization
//! - Project up/down commands
//! - Docker Compose file generation

pub mod commands;
pub mod compose_generator;
pub mod config;
pub mod init;
pub mod registry;
