use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "cert")]
#[command(about = "Certificate management tool using mkcert", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install mkcert and setup local CA
    InstallCa,
    /// Generate certificate for a project domain
    Generate {
        /// Domain name to generate certificate for
        domain: String,
        /// Certificate directory
        #[arg(short, long, default_value = "./caddy/certs")]
        cert_dir: PathBuf,
    },
    /// List all generated certificates
    List {
        /// Certificate directory
        #[arg(short, long, default_value = "./caddy/certs")]
        cert_dir: PathBuf,
    },
    /// Remove certificate for a domain
    Remove {
        /// Domain name to remove certificate for
        domain: String,
        /// Certificate directory
        #[arg(short, long, default_value = "./caddy/certs")]
        cert_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InstallCa => install_ca()?,
        Commands::Generate { domain, cert_dir } => generate_cert(&domain, &cert_dir)?,
        Commands::List { cert_dir } => list_certs(&cert_dir)?,
        Commands::Remove { domain, cert_dir } => remove_cert(&domain, &cert_dir)?,
    }

    Ok(())
}

fn install_ca() -> Result<()> {
    println!("{}", "Installing mkcert...".blue());

    // Check if mkcert is installed
    let mkcert_installed = Command::new("mkcert").arg("-version").output().is_ok();

    if !mkcert_installed {
        println!(
            "{}",
            "mkcert not found. Installing via Homebrew...".yellow()
        );

        let status = Command::new("brew")
            .args(&["install", "mkcert"])
            .status()
            .context("Failed to run brew. Make sure Homebrew is installed.")?;

        if !status.success() {
            anyhow::bail!("Failed to install mkcert via Homebrew");
        }
    }

    println!("{}", "Installing local CA...".blue());
    let status = Command::new("mkcert")
        .arg("-install")
        .status()
        .context("Failed to install local CA")?;

    if !status.success() {
        anyhow::bail!("Failed to install local CA");
    }

    println!();
    println!("{}", "✓ Local CA installed successfully".green());
    println!();
    println!("Next steps:");
    println!("  1. Generate certificates: cert generate <domain>");
    println!("  2. Add domains to /etc/hosts");

    Ok(())
}

fn generate_cert(domain: &str, cert_dir: &Path) -> Result<()> {
    println!(
        "{} {}",
        "Generating certificate for".blue(),
        domain.bright_white()
    );

    // Create cert directory if it doesn't exist
    std::fs::create_dir_all(cert_dir).context("Failed to create certificate directory")?;

    // Generate certificate with wildcard
    let output = Command::new("mkcert")
        .current_dir(cert_dir)
        .args(&[domain, &format!("*.{}", domain)])
        .output()
        .context("Failed to run mkcert")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to generate certificate: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Rename files to standard naming
    let source_cert = cert_dir.join(format!("{}+1.pem", domain));
    let source_key = cert_dir.join(format!("{}+1-key.pem", domain));
    let dest_cert = cert_dir.join(format!("{}.crt", domain));
    let dest_key = cert_dir.join(format!("{}.key", domain));

    std::fs::rename(&source_cert, &dest_cert).context("Failed to rename certificate file")?;
    std::fs::rename(&source_key, &dest_key).context("Failed to rename key file")?;

    println!();
    println!("{}", "✓ Certificate generated successfully".green());
    println!("  Certificate: {}", dest_cert.display());
    println!("  Private Key: {}", dest_key.display());
    println!();
    println!("Add to /etc/hosts:");
    println!("  127.0.0.1 {} *.{}", domain, domain);

    Ok(())
}

fn list_certs(cert_dir: &Path) -> Result<()> {
    println!("{} {}", "📋 Certificates in".blue(), cert_dir.display());
    println!();

    if !cert_dir.exists() {
        println!("{}", "No certificates directory found".yellow());
        return Ok(());
    }

    let entries = std::fs::read_dir(cert_dir).context("Failed to read certificate directory")?;

    let mut found = false;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "crt" {
                if let Some(stem) = path.file_stem() {
                    println!("  • {}", stem.to_string_lossy());
                    found = true;
                }
            }
        }
    }

    if !found {
        println!("{}", "No certificates found".yellow());
    }

    Ok(())
}

fn remove_cert(domain: &str, cert_dir: &Path) -> Result<()> {
    let cert_file = cert_dir.join(format!("{}.crt", domain));
    let key_file = cert_dir.join(format!("{}.key", domain));

    let mut removed = false;

    if cert_file.exists() {
        std::fs::remove_file(&cert_file).context("Failed to remove certificate file")?;
        removed = true;
    }

    if key_file.exists() {
        std::fs::remove_file(&key_file).context("Failed to remove key file")?;
        removed = true;
    }

    if removed {
        println!("{} {}", "✓ Certificate removed for".green(), domain);
    } else {
        println!("{} {}", "⚠ No certificate found for".yellow(), domain);
    }

    Ok(())
}
