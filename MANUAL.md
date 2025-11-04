# oh-my-dockers Manual

Complete reference guide for the oh-my-dockers CLI tool.

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Command Reference](#command-reference)
5. [Project Configuration](#project-configuration)
6. [Advanced Usage](#advanced-usage)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

## Introduction

oh-my-dockers is a comprehensive CLI tool for managing Docker development environments. It provides:

- **Unified Management**: Single tool for networks, proxies, ports, and projects
- **Automatic Configuration**: Generates Docker Compose files and Caddy configurations
- **Port Management**: Visualize and plan port mappings across all networks
- **HTTPS Support**: Local SSL certificate management for secure development

## Installation

### System Requirements

- **Operating System**: macOS, Linux, or Windows (WSL2)
- **Docker**: Version 20.10 or later
- **Docker Compose**: Version 2.0 or later
- **Rust**: 1.70 or later (for building from source)
- **Caddy**: Latest version (for reverse proxy)

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd oh-my-dockers

# Build the release binary
cargo build --release

# The binary will be at target/release/oh-my-dockers
```

### Installing to System PATH

```bash
# Install globally
cargo install --path .

# Or add to your PATH manually
export PATH="$PATH:$(pwd)/target/release"
```

## Configuration

### Configuration Directory

The tool uses `~/.oh-my-dockers` as the default configuration directory. This directory is automatically created on first run.

**Custom Configuration Directory**

Set the `OH_MY_DOCKERS_DIR` environment variable:

```bash
export OH_MY_DOCKERS_DIR="/path/to/config"
oh-my-dockers project list
```

### Global Configuration

The global configuration file is located at `~/.oh-my-dockers/config.toml`:

```toml
[global]
caddy_network = "caddy-net"
projects_dir = "projects"
templates_dir = "templates"
init_dir = "init"
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
postgres_version = "latest"
redis_version = "latest"
surrealdb_version = "latest"
chroma_version = "latest"
ollama_version = "latest"
n8n_version = "latest"
timezone = "Asia/Tokyo"
```

### Directory Structure

```
~/.oh-my-dockers/
├── config.toml              # Global configuration
├── projects/                # Project TOML files
│   ├── my-project.toml
│   └── ...
├── caddy/                   # Caddy reverse proxy
│   ├── Caddyfile            # Main Caddyfile
│   ├── certs/               # SSL certificates (.crt, .key)
│   └── projects/             # Project-specific configs (*.caddy)
├── templates/               # Docker Compose service templates
│   ├── postgres.yml
│   ├── redis.yml
│   └── ...
├── init/                    # Initialization scripts
│   ├── postgres/
│   └── redis/
└── generated/               # Generated Docker Compose files
    └── docker-compose-*.yml
```

## Command Reference

### Network Management

#### `network list`

List all Docker networks.

```bash
oh-my-dockers network list
```

**Output Format:**
```
Docker Networks:

  NAME                           DRIVER          SCOPE
  ------------------------------------------------------------
  bridge                         bridge          local
  caddy-net                      bridge          local
  my-network                     bridge          local
```

#### `network create <name>`

Create a new Docker network.

```bash
oh-my-dockers network create my-network
```

**Options:**
- `name`: Name of the network to create

**Example:**
```bash
oh-my-dockers network create dev-network
```

#### `network remove <name>`

Remove a Docker network.

```bash
oh-my-dockers network remove my-network
```

**Options:**
- `name`: Name of the network to remove

**Note:** Networks with connected containers cannot be removed.

#### `network connect <network> <container>`

Connect a container to a network.

```bash
oh-my-dockers network connect my-network my-container
```

**Options:**
- `network`: Network name
- `container`: Container name or ID

### Reverse Proxy Management

#### `proxy add <domain> <target>`

Add a reverse proxy rule.

```bash
oh-my-dockers proxy add example.com backend:8080
```

**Options:**
- `domain`: Domain name (e.g., `example.com` or `api.example.com`)
- `target`: Backend target (e.g., `container:port` or `host:port`)

**Example:**
```bash
# Simple domain
oh-my-dockers proxy add myapp.local app:3000

# Subdomain
oh-my-dockers proxy add api.myapp.local backend:8080
```

**Notes:**
- Creates a Caddy configuration file automatically
- Requires SSL certificates in `~/.oh-my-dockers/caddy/certs/`
- Automatically reloads Caddy if running

#### `proxy list`

List all configured proxy rules.

```bash
oh-my-dockers proxy list
```

**Output Format:**
```
Proxy Rules:

  DOMAIN                                   TARGET
  ------------------------------------------------------------
  example.com                             backend:8080
  api.example.com                         api:3000
```

#### `proxy remove <domain>`

Remove a proxy rule.

```bash
oh-my-dockers proxy remove example.com
```

**Options:**
- `domain`: Domain name to remove

#### `proxy reload`

Reload Caddy configuration.

```bash
oh-my-dockers proxy reload
```

**Note:** Only works if Caddy container (`oh-my-dockers-caddy`) is running.

### Port Mapping Display

#### `ports`

List all port mappings across all networks.

```bash
oh-my-dockers ports
```

**Output Format:**
```
Port Mappings:

  Network: bridge
  ------------------------------------------------------------
  CONTAINER                  INTERNAL         LOCAL           PROTOCOL
  ------------------------------------------------------------
  postgres-container         5432             5432           tcp
  redis-container           6379             6379           tcp

  Network: my-network
  ------------------------------------------------------------
  CONTAINER                  INTERNAL         LOCAL           PROTOCOL
  ------------------------------------------------------------
  web-app                    80               8080           tcp
```

#### `ports show <network>`

Show port mappings for a specific network.

```bash
oh-my-dockers ports show my-network
```

**Options:**
- `network`: Network name

**Example:**
```bash
oh-my-dockers ports show caddy-net
```

### Project Management

#### `project list`

List all configured projects.

```bash
oh-my-dockers project list
```

**Output Format:**
```
Available projects:

  • my-project
    Domain: my-project.local
    Mode: managed

  • api-project
    Domain: api.local
    Mode: proxy-only
```

#### `project up <project>`

Start a project.

```bash
oh-my-dockers project up my-project
```

**Options:**
- `project`: Project name (corresponds to TOML file name)

**What it does:**
1. Loads project configuration
2. Creates/ensures networks exist
3. Generates Caddy configuration
4. Generates Docker Compose file (if managed mode)
5. Starts Docker Compose services (if managed mode)
6. Connects Caddy to project network
7. Reloads Caddy configuration

**Example:**
```bash
oh-my-dockers project up daily
```

#### `project down <project>`

Stop a project.

```bash
oh-my-dockers project down my-project
```

**Options:**
- `project`: Project name

**What it does:**
1. Stops Docker Compose services (if managed mode)
2. Removes Caddy configuration
3. Reloads Caddy (if running)

**Example:**
```bash
oh-my-dockers project down daily
```

### Migration

#### `migrate`

Migrate existing configuration from current directory to `~/.oh-my-dockers`.

```bash
oh-my-dockers migrate
```

**What it migrates:**
- `projects/` directory → `~/.oh-my-dockers/projects/`
- `templates/` directory → `~/.oh-my-dockers/templates/`
- `init/` directory → `~/.oh-my-dockers/init/`
- `caddy/` directory → `~/.oh-my-dockers/caddy/`
- `config.toml` → `~/.oh-my-dockers/config.toml` (with path updates)

**Usage:**
Run from the directory containing your existing configuration:

```bash
cd /path/to/old/config
oh-my-dockers migrate
```

## Project Configuration

### Configuration File Format

Project configuration files use TOML format and are stored in `~/.oh-my-dockers/projects/<project-name>.toml`.

### Basic Structure

```toml
[project]
name = "project-name"
domain = "project.local"
mode = "managed"  # or "proxy-only"
port_offset = 0  # Optional: offset for database ports

[services]
postgres = { enabled = true, version = "latest" }
redis = { enabled = true, version = "latest" }

[network]
name = "project-net"

[caddy]
auto_subdomains = true
routes = []  # Optional: custom routes
```

### Project Section

```toml
[project]
name = "my-project"           # Project name (must match filename)
domain = "my-project.local"  # Base domain for the project
mode = "managed"             # "managed" or "proxy-only"
port_offset = 0              # Optional: port offset for databases
```

**Fields:**
- `name`: Project identifier (must match the TOML filename without extension)
- `domain`: Base domain name (used for subdomains and SSL certificates)
- `mode`: 
  - `managed`: oh-my-dockers manages all services via Docker Compose
  - `proxy-only`: Only provides reverse proxy, services managed externally
- `port_offset`: Optional offset for database ports to avoid conflicts
  - Example: `port_offset = 100` makes PostgreSQL use port 5532 instead of 5432

### Services Section

```toml
[services]
postgres = { enabled = true, version = "16" }
redis = { enabled = true, version = "7" }
n8n = { enabled = true, version = "latest" }
chroma = { enabled = false, version = "latest" }
```

**Available Services:**
- `postgres`: PostgreSQL database
- `redis`: Redis cache
- `surrealdb`: SurrealDB database
- `chroma`: Chroma vector database
- `ollama`: Ollama LLM server
- `n8n`: n8n workflow automation

**Service Configuration:**
- `enabled`: Boolean to enable/disable the service
- `version`: Docker image version tag

### Network Section

```toml
[network]
name = "my-project-net"
external = false  # Optional: use existing network
```

**Fields:**
- `name`: Network name (will be created if doesn't exist)
- `external`: Set to `true` if network already exists externally

### Caddy Section

```toml
[caddy]
auto_subdomains = true
routes = [
    { subdomain = "api", target = "backend:3000" },
    { domain = "custom.local", target = "service:80" }
]
```

**Fields:**
- `auto_subdomains`: Automatically create subdomains for enabled HTTP services
- `routes`: Custom reverse proxy routes

**Route Configuration:**
- `subdomain`: Subdomain under the project domain (e.g., `api` → `api.project.local`)
- `domain`: Full domain name
- `target`: Backend target (format: `container:port` or `host:port`)

**Example Routes:**
```toml
routes = [
    # Subdomain route
    { subdomain = "api", target = "backend:3000" },
    
    # Full domain route
    { domain = "custom.example.com", target = "service:8080" }
]
```

### Managed Mode Example

```toml
[project]
name = "daily"
domain = "daily.local"
mode = "managed"
port_offset = 0

[services]
postgres = { enabled = true, version = "latest" }
redis = { enabled = true, version = "latest" }
n8n = { enabled = true, version = "latest" }

[network]
name = "daily-net"

[caddy]
auto_subdomains = true
```

### Proxy-Only Mode Example

```toml
[project]
name = "api"
domain = "api.local"
mode = "proxy-only"

[network]
name = "api-net"
external = true

[caddy]
auto_subdomains = false
routes = [
    { subdomain = "api", target = "backend:3000" },
    { subdomain = "admin", target = "admin:8080" }
]
```

## Advanced Usage

### Multiple Projects with Port Offsets

When running multiple projects simultaneously, use port offsets to avoid conflicts:

**Project 1 (`daily.toml`):**
```toml
[project]
name = "daily"
port_offset = 0  # PostgreSQL: 5432, Redis: 6379
```

**Project 2 (`test.toml`):**
```toml
[project]
name = "test"
port_offset = 100  # PostgreSQL: 5532, Redis: 6479
```

**Project 3 (`staging.toml`):**
```toml
[project]
name = "staging"
port_offset = 200  # PostgreSQL: 5632, Redis: 6579
```

### Custom Templates

Create custom service templates in `~/.oh-my-dockers/templates/`:

**Example: `~/.oh-my-dockers/templates/custom-service.yml`**
```yaml
services:
  custom-service:
    image: custom-image:${CUSTOM_VERSION:-latest}
    environment:
      - PROJECT_NAME=${PROJECT_NAME}
    networks:
      - ${PROJECT_NETWORK}
```

**Usage in Project:**
```toml
[services]
custom-service = { enabled = true, version = "1.0" }
```

### Environment Variables

Projects can include `.env` files in `~/.oh-my-dockers/projects/<project>/.env`:

```bash
# ~/.oh-my-dockers/projects/my-project/.env
POSTGRES_PASSWORD=secret123
REDIS_PASSWORD=secret456
```

These variables are available in templates via `${VAR_NAME}`.

### SSL Certificates

Place SSL certificates in `~/.oh-my-dockers/caddy/certs/`:

**Certificate Naming:**
- Certificate: `<domain>.crt` (replace dots with underscores)
- Key: `<domain>.key` (replace dots with underscores)

**Example:**
- Domain: `example.com`
- Certificate: `example_com.crt`
- Key: `example_com.key`

**Generating Self-Signed Certificates:**

```bash
# Generate certificate for example.com
openssl req -x509 -newkey rsa:4096 -keyout ~/.oh-my-dockers/caddy/certs/example_com.key \
  -out ~/.oh-my-dockers/caddy/certs/example_com.crt -days 365 -nodes \
  -subj "/CN=example.com"
```

## Troubleshooting

### Caddy Not Running

**Problem:** `proxy reload` fails with "Caddy is not running"

**Solution:** Start Caddy container:

```bash
docker-compose -f docker-compose-caddy.yml up -d
```

### Port Conflicts

**Problem:** Port already in use when starting project

**Solution:** 
1. Check what's using the port: `lsof -i :5432`
2. Use `port_offset` in project configuration
3. Or stop conflicting services

### Network Already Exists

**Problem:** `network create` fails because network exists

**Solution:** This is normal - the tool checks and skips creation if exists. Use `network list` to verify.

### Certificate Errors

**Problem:** HTTPS not working, certificate errors

**Solution:**
1. Ensure certificates exist in `~/.oh-my-dockers/caddy/certs/`
2. Check certificate naming matches domain (dots → underscores)
3. Verify Caddy has read access to certs directory

### Project Not Found

**Problem:** `project up my-project` fails with "project not found"

**Solution:**
1. Check project file exists: `ls ~/.oh-my-dockers/projects/my-project.toml`
2. Verify filename matches project name
3. Check TOML syntax is valid

### Migration Issues

**Problem:** Migration doesn't work

**Solution:**
1. Run from directory containing old config
2. Check `OH_MY_DOCKERS_DIR` is not set incorrectly
3. Verify source directories exist

## Best Practices

### Project Organization

1. **Use descriptive names**: Project names should reflect their purpose
2. **One project per domain**: Each project should have a unique domain
3. **Version pinning**: Pin service versions in production-like environments
4. **Port planning**: Use `ports` command to plan port assignments

### Network Management

1. **Isolated networks**: Create separate networks for each project
2. **Naming conventions**: Use consistent naming (e.g., `<project>-net`)
3. **External networks**: Mark external networks explicitly

### Security

1. **SSL certificates**: Always use HTTPS for sensitive services
2. **Environment variables**: Store secrets in `.env` files (not in TOML)
3. **Network isolation**: Use separate networks for different projects

### Performance

1. **Port offsets**: Use offsets to run multiple projects simultaneously
2. **Resource limits**: Add resource limits in templates for production
3. **Volume management**: Use named volumes for persistent data

### Maintenance

1. **Regular cleanup**: Remove unused networks and proxy rules
2. **Configuration backup**: Backup `~/.oh-my-dockers` regularly
3. **Version updates**: Keep service versions updated

## Examples

### Complete Workflow Example

```bash
# 1. List available projects
oh-my-dockers project list

# 2. Check networks
oh-my-dockers network list

# 3. Start a project
oh-my-dockers project up daily

# 4. View port mappings
oh-my-dockers ports

# 5. Add a custom proxy rule
oh-my-dockers proxy add api.daily.local backend:3000

# 6. Check proxy rules
oh-my-dockers proxy list

# 7. Stop the project
oh-my-dockers project down daily
```

### Multi-Project Setup

```bash
# Start multiple projects
oh-my-dockers project up daily      # port_offset = 0
oh-my-dockers project up staging    # port_offset = 100
oh-my-dockers project up testing    # port_offset = 200

# View all port mappings
oh-my-dockers ports

# Stop all projects
oh-my-dockers project down daily
oh-my-dockers project down staging
oh-my-dockers project down testing
```

## See Also

- [README.md](README.md) - Project overview and quick start
- [TEST_RESULTS.md](TEST_RESULTS.md) - Test results and verification

