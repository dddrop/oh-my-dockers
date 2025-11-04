# oh-my-dockers

A powerful CLI tool for managing Docker development environments with automatic reverse proxy configuration and network management.

## Features

- 🚀 **Project Management**: Start and stop Docker Compose projects with a single command
- 🌐 **Network Management**: Create, list, and manage Docker networks
- 🔄 **Reverse Proxy**: Automatic HTTPS reverse proxy configuration with Caddy
- 🔌 **Port Mapping**: View and manage port mappings across Docker networks
- 📁 **Configuration Management**: Centralized configuration in `~/.oh-my-dockers`
- 🔐 **HTTPS Support**: Local SSL certificate support for secure development
- 🎯 **Port Planning**: Visualize all network port mappings in one place

## Installation

### Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- Caddy (for reverse proxy)

### Build from Source

```bash
git clone <repository-url>
cd oh-my-dockers
cargo build --release
```

The binary will be available at `target/release/oh-my-dockers`.

### Install to System Path (Optional)

```bash
cargo install --path .
```

## Quick Start

### 1. First Run

The tool automatically creates the configuration directory on first run:

```bash
oh-my-dockers project list
```

Configuration will be created at `~/.oh-my-dockers` (or `$OH_MY_DOCKERS_DIR` if set).

### 2. Migrate Existing Configuration

If you have existing configuration files in the project directory:

```bash
oh-my-dockers migrate
```

This will migrate your projects, templates, and Caddy configurations to the new location.

### 3. Create a Project

Create a project configuration file at `~/.oh-my-dockers/projects/my-project.toml`:

```toml
[project]
name = "my-project"
domain = "my-project.local"
mode = "managed"

[network]
name = "my-project-net"

[caddy]
auto_subdomains = true
```

### 4. Start a Project

```bash
oh-my-dockers project up my-project
```

### 5. View Port Mappings

```bash
oh-my-dockers ports
```

## Configuration

### Configuration Directory

By default, configuration is stored in `~/.oh-my-dockers`. You can customize this by setting the `OH_MY_DOCKERS_DIR` environment variable:

```bash
export OH_MY_DOCKERS_DIR="/custom/path"
```

### Directory Structure

```
~/.oh-my-dockers/
├── config.toml          # Global configuration
├── projects/             # Project configuration files
├── caddy/                # Caddy configuration
│   ├── Caddyfile
│   ├── certs/            # SSL certificates
│   └── projects/         # Project-specific Caddy configs
├── templates/            # Docker Compose templates
├── init/                # Initialization scripts
└── generated/           # Generated Docker Compose files
```

## Basic Usage

### Network Management

```bash
# List all networks
oh-my-dockers network list

# Create a network
oh-my-dockers network create my-network

# Remove a network
oh-my-dockers network remove my-network

# Connect a container to a network
oh-my-dockers network connect my-network my-container
```

### Reverse Proxy Management

```bash
# Add a proxy rule
oh-my-dockers proxy add example.com backend:8080

# List all proxy rules
oh-my-dockers proxy list

# Remove a proxy rule
oh-my-dockers proxy remove example.com

# Reload Caddy configuration
oh-my-dockers proxy reload
```

### Port Mapping

```bash
# List all port mappings
oh-my-dockers ports

# Show ports for a specific network
oh-my-dockers ports show my-network
```

### Project Management

```bash
# List all projects
oh-my-dockers project list

# Start a project
oh-my-dockers project up my-project

# Stop a project
oh-my-dockers project down my-project
```

## Project Configuration

Projects are configured using TOML files in `~/.oh-my-dockers/projects/`.

### Managed Mode

In managed mode, oh-my-dockers controls all services:

```toml
[project]
name = "my-project"
domain = "my-project.local"
mode = "managed"
port_offset = 0  # Optional: offset for database ports

[services]
postgres = { enabled = true, version = "latest" }
redis = { enabled = true, version = "latest" }
n8n = { enabled = true, version = "latest" }

[network]
name = "my-project-net"

[caddy]
auto_subdomains = true
```

### Proxy-Only Mode

In proxy-only mode, services are managed externally:

```toml
[project]
name = "my-project"
domain = "my-project.local"
mode = "proxy-only"

[network]
name = "my-project-net"
external = true

[caddy]
routes = [
    { subdomain = "api", target = "backend:3000" },
    { subdomain = "app", target = "frontend:80" }
]
```

## Requirements

- Docker and Docker Compose installed and running
- Caddy container running (for reverse proxy features)
- Rust (for building from source)

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

## See Also

- [MANUAL.md](MANUAL.md) - Detailed usage manual
- [TEST_RESULTS.md](TEST_RESULTS.md) - Test results and verification
