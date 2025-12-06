//! Caddy configuration generation for projects
//!
//! This module handles generating Caddy reverse proxy configurations
//! for projects based on their docker-compose.yml files.

use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{get_config_dir, load_global_config};
use crate::docker::compose::ComposeInfo;
use crate::project::config::ProjectConfig;

/// Generate mkcert certificate for a project (main domain + wildcard)
fn generate_project_certificate(
    base_domain: &str,
    cert_file: &std::path::Path,
    key_file: &std::path::Path,
) -> Result<()> {
    println!(
        "{} Generating mkcert certificate for {} and *.{}...",
        "ℹ".blue(),
        base_domain.bright_white(),
        base_domain
    );

    // Check if mkcert is available
    let mkcert_path = Command::new("which")
        .arg("mkcert")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    let mkcert = mkcert_path.as_deref().unwrap_or("mkcert");

    // Generate certificate with both main domain and wildcard
    let wildcard = format!("*.{}", base_domain);
    let output = Command::new(mkcert)
        .arg(base_domain)
        .arg(&wildcard)
        .output()
        .context("Failed to run mkcert. Make sure mkcert is installed.")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("mkcert failed: {}", error);
    }

    // Find the generated certificate files
    // mkcert generates files like: domain+1.pem and domain+1-key.pem
    let current_dir = std::env::current_dir()?;
    let base_domain_str = base_domain.to_string();

    let mut cert_files: Vec<_> = current_dir
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.contains(&base_domain_str) && name.ends_with(".pem") && !name.contains("-key")
        })
        .collect();

    let mut key_files: Vec<_> = current_dir
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.contains(&base_domain_str) && name.ends_with("-key.pem")
        })
        .collect();

    cert_files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
    key_files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));

    if cert_files.is_empty() || key_files.is_empty() {
        anyhow::bail!("Failed to find generated certificate files");
    }

    let latest_cert = cert_files.last().unwrap();
    let latest_key = key_files.last().unwrap();

    // Copy certificate files to target location
    fs::copy(latest_cert.path(), cert_file).context("Failed to copy certificate file")?;
    fs::copy(latest_key.path(), key_file).context("Failed to copy key file")?;

    // Clean up temporary files
    let _ = fs::remove_file(latest_cert.path());
    let _ = fs::remove_file(latest_key.path());

    println!(
        "{} Certificate generated: {} and {}",
        "✓".green(),
        cert_file.display(),
        key_file.display()
    );

    Ok(())
}

/// Generate Caddy configuration for a project
pub fn generate_caddy_config(config: &ProjectConfig, compose_info: &ComposeInfo) -> Result<()> {
    println!("{} Generating Caddy configuration...", "ℹ".blue());

    let config_dir = get_config_dir()?;
    let global_config = load_global_config()?;
    let output_dir = config_dir.join(&global_config.global.caddy_projects_dir);
    fs::create_dir_all(&output_dir).context("Failed to create caddy projects directory")?;

    let output_file = output_dir.join(format!("{}.caddy", config.project.name));
    let mut caddy_config = format!(
        "# Auto-generated Caddy configuration for {}\n# Domain: {}\n\n",
        config.project.name, config.project.domain
    );

    // Check if HTTPS is enabled in global config
    let enable_https = global_config.global.enable_https;

    // Generate project-level certificate if needed (main domain + wildcard)
    let project_cert_name = config.project.domain.replace('.', "_");
    let project_cert_file = config_dir
        .join(&global_config.global.caddy_certs_dir)
        .join(format!("{}.crt", project_cert_name));
    let project_key_file = config_dir
        .join(&global_config.global.caddy_certs_dir)
        .join(format!("{}.key", project_cert_name));

    if enable_https && (!project_cert_file.exists() || !project_key_file.exists()) {
        // Generate project certificate (main domain + wildcard)
        if let Err(e) = generate_project_certificate(
            &config.project.domain,
            &project_cert_file,
            &project_key_file,
        ) {
            println!(
                "{} Failed to generate project certificate: {}",
                "⚠".yellow(),
                e
            );
            println!(
                "{} Falling back to Caddy's internal certificate",
                "ℹ".blue()
            );
        }
    }

    // Helper function to get TLS configuration for a domain
    // All domains use the same project certificate
    let get_tls_config = |_domain: &str| -> Result<String> {
        if !enable_https {
            return Ok(String::new());
        }

        if project_cert_file.exists() && project_key_file.exists() {
            // Use project certificate (works for all subdomains)
            Ok(format!(
                "    tls /certs/{}.crt /certs/{}.key\n",
                project_cert_name, project_cert_name
            ))
        } else {
            // Fall back to Caddy's internal certificate
            Ok("    tls internal\n".to_string())
        }
    };

    // Generate routes based on user configuration
    if !config.caddy.routes.is_empty() {
        println!("{} Adding custom routes...", "ℹ".blue());

        for (subdomain, target) in &config.caddy.routes {
            let full_domain = format!("{}.{}", subdomain, config.project.domain);

            let tls_config = get_tls_config(&full_domain)?;
            caddy_config.push_str(&format!(
                "{} {{\n{}    reverse_proxy {}\n}}\n\n",
                full_domain, tls_config, target
            ));

            println!("  {} -> {}", full_domain.bright_white(), target);
        }
    } else {
        // Auto-generate routes from docker-compose services
        println!(
            "{} Auto-generating routes from docker-compose.yml...",
            "ℹ".blue()
        );

        for (service_name, service_info) in &compose_info.services {
            // Skip services without container ports (like databases without HTTP interface)
            if service_info.container_ports.is_empty() {
                continue;
            }

            // Use the first container port as default
            let port = service_info.container_ports[0];

            // Determine container name
            let container_name = service_info
                .container_name
                .clone()
                .unwrap_or_else(|| format!("{}-{}-1", config.project.name, service_name));

            let subdomain = service_name;
            let full_domain = format!("{}.{}", subdomain, config.project.domain);
            let target = format!("{}:{}", container_name, port);

            let tls_config = get_tls_config(&full_domain)?;
            caddy_config.push_str(&format!(
                "{} {{\n{}    reverse_proxy {}\n}}\n\n",
                full_domain, tls_config, target
            ));

            println!("  {} -> {}", full_domain.bright_white(), target);
        }
    }

    fs::write(&output_file, caddy_config).context("Failed to write Caddy configuration")?;

    println!("{} Generated {:?}", "✓".green(), output_file);

    Ok(())
}
