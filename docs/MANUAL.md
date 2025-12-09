# omd Manual

Complete reference guide for the omd CLI tool.

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [Command Reference](#command-reference)
6. [Project Configuration](#project-configuration)
7. [Advanced Usage](#advanced-usage)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)

## Introduction

omd (oh-my-dockers) is a comprehensive CLI tool for managing Docker development environments. It provides:

- **Local Project Configuration**: Each project manages its own `omd.toml` configuration
- **Automatic Network Management**: Creates and manages Docker networks automatically
- **Port Conflict Detection**: Prevents port conflicts across multiple projects
- **Reverse Proxy Integration**: Generates Caddy configurations for HTTPS access
- **Smart Container Discovery**: Automatically parses `docker-compose.yml` for container information
- **Centralized Registry**: Tracks all projects and their port allocations

### Key Concepts

**Project-Based Workflow**: Unlike centralized configuration systems, omd works with local project directories. Each project has its own `omd.toml` configuration file.

**Port Registry**: omd maintains a global registry (`~/.oh-my-dockers/registry.json`) tracking which ports are used by which projects, preventing conflicts.

**Caddy Integration**: Automatically generates reverse proxy configurations, allowing you to access services via HTTPS with custom domains.

## Installation

### System Requirements

- **Operating System**: macOS, Linux, or Windows (WSL2)
- **Docker**: Version 20.10 or later
- **Docker Compose**: Version 2.0 or later
- **Rust**: 1.70 or later (for building from source)
- **Caddy**: Latest version (for reverse proxy features)

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd oh-my-dockers

# Build the release binary
cargo build --release

# The binary will be at target/release/omd
```

### Installing to System PATH

```bash
# Install globally
cargo install --path .

# Verify installation
omd --help
```

## Quick Start

### Basic Workflow

```bash
# 1. Navigate to your project
cd /path/to/your/project

# 2. Initialize omd configuration
omd init

# 3. Create or ensure docker-compose.yml exists
# (your existing docker-compose.yml file)

# 4. Configure infrastructure and start containers
omd project up

# 5. Access your services
# https://your-project.local
```

### Complete Example

```bash
# Create a new project
mkdir my-api
cd my-api

# Initialize omd
omd init
# Project name [my-api]: 
# Domain [my-api.local]: 
# Network name [my-api-net]: 
# Do you want to configure Caddy routes now? [y/N]: n

# Create docker-compose.yml
cat > docker-compose.yml <<EOF
services:
  postgres:
    image: postgres:15
    container_name: my-api-postgres
    ports:
      - "5432:5432"
    environment:
      POSTGRES_PASSWORD: secret
    networks:
      - my-api-net

  api:
    build: .
    container_name: my-api-backend
    ports:
      - "3000:3000"
    depends_on:
      - postgres
    networks:
      - my-api-net

networks:
  my-api-net:
EOF

# Configure infrastructure and start containers
omd project up
# ℹ Parsing docker-compose.yml...
# ℹ Found host ports: 5432, 3000
# ✓ No port conflicts
# ✓ Network created
# ✓ Caddy configuration generated
# ✓ Project my-api is configured!
# ℹ Starting containers...
# ✓ Containers started successfully

# Access at https://api.my-api.local
```

## Configuration

### Configuration Directory

The tool uses `~/.oh-my-dockers` as the default configuration directory.

**Directory Structure:**

```
~/.oh-my-dockers/
├── config.toml          # Global settings
├── registry.json        # Project registry with port allocations
└── caddy/
    ├── Caddyfile        # Main Caddy config
    ├── certs/           # SSL certificates
    └── projects/        # Generated per-project Caddy configs
```

**Custom Configuration Directory:**

```bash
export OH_MY_DOCKERS_DIR="/custom/path"
```

### Global Configuration

The global configuration file is located at `~/.oh-my-dockers/config.toml`:

```toml
[global]
# Caddy network name
caddy_network = "caddy-net"

# Directories (relative to config directory)
caddy_projects_dir = "caddy/projects"
caddy_certs_dir = "caddy/certs"

[defaults]
# Default timezone
timezone = "Asia/Tokyo"

# Network definitions
[networks]
# Caddy reverse proxy network
caddy-net = {}
```

**Configuration Options:**

- `caddy_network`: Name of the Caddy network (default: "caddy-net")
- `caddy_projects_dir`: Directory for project-specific Caddy configs
- `caddy_certs_dir`: Directory for SSL certificates
- `timezone`: Default timezone for services

### Project Registry

The project registry (`~/.oh-my-dockers/registry.json`) tracks all registered projects:

```json
{
  "projects": {
    "my-api": {
      "name": "my-api",
      "path": "/path/to/my-api",
      "domain": "my-api.local",
      "network": "my-api-net",
      "ports": [5432, 3000],
      "containers": ["my-api-postgres", "my-api-backend"]
    }
  }
}
```

**Warning**: Don't edit this file manually. Use `omd up` and `omd down` to manage registrations.

## Command Reference

### omd caddy start

Start the Caddy reverse proxy container.

```bash
omd caddy start
```

**Note:** Caddy is usually auto-started when you run `omd project up`. You only need to manually start it if you've stopped it or want to ensure it's running.

### omd caddy stop

Stop the Caddy container.

```bash
omd caddy stop
```

### omd caddy restart

Restart the Caddy container.

```bash
omd caddy restart
```

### omd caddy status

Show Caddy container status and connection information.

```bash
omd caddy status
```

**Example Output:**
```
Caddy Status:

  Status: Running
  Up 2 hours    0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp

Admin API: http://localhost:2019
Logs: docker logs oh-my-dockers-caddy -f
```

### omd caddy logs

Show Caddy container logs.

```bash
# Show recent logs
omd caddy logs

# Follow logs in real-time
omd caddy logs --follow
```

### omd init

Initialize `omd.toml` configuration in the current directory.

```bash
omd init
```

**Interactive Prompts:**
- Project name (default: current directory name)
- Domain (default: `{project-name}.local`)
- Network name (default: `{project-name}-net`)
- Configure Caddy routes (optional)

**Output**: Creates `omd.toml` in current directory.

### omd project up

Configure project infrastructure and start containers (run from project directory).

```bash
cd /path/to/project
omd project up
```

**What it does:**
1. Reads `omd.toml` from current directory
2. Parses `docker-compose.yml` to extract ports and container names
3. Checks for port conflicts with other registered projects
4. Creates Docker networks if they don't exist
5. **Automatically starts Caddy** if not running
6. Generates Caddy reverse proxy configuration
7. Registers project in global registry
8. **Starts containers** (`docker compose up -d`)

**Example Output:**

```
ℹ Project: my-api
ℹ Domain: my-api.local
ℹ Network: my-api-net
ℹ Parsing docker-compose.yml...
ℹ Found host ports: 5432, 3000
ℹ Container names: my-api-postgres, my-api-backend
✓ No port conflicts
✓ Network created
✓ Caddy configuration generated
✓ Project my-api is configured!

ℹ Starting containers...
✓ Containers started successfully

Access your project at: https://my-api.local
```

### omd project down

Stop containers (run from project directory).

```bash
cd /path/to/project
omd project down
```

**What it does:**
1. Reads `omd.toml` from current directory
2. Stops containers (`docker compose down`)

**Note**: This only stops containers. Configuration remains intact. Use `omd project remove` to also remove configuration.

### omd project remove

Stop containers and remove all project configuration (run from project directory).

```bash
cd /path/to/project
omd project remove
```

**What it does:**
1. Reads `omd.toml` from current directory
2. Stops containers (`docker compose down`)
3. Removes Caddy configuration
4. Unregisters project from global registry
5. Removes domains from `/etc/hosts`
6. Reloads Caddy

### omd project list

List all registered projects.

```bash
omd project list
```

**Example Output:**

```
Registered projects:

  • my-api
    Path: /Users/dev/projects/my-api
    Domain: my-api.local
    Network: my-api-net
    Ports: 5432, 3000

  • my-web
    Path: /Users/dev/projects/my-web
    Domain: my-web.local
    Network: my-web-net
    Ports: 8080, 8443
```

### omd network list

List all Docker networks.

```bash
omd network list
```

**Example Output:**

```
Docker Networks:

  • bridge (172.17.0.0/16)
    Driver: bridge

  • caddy-net (172.20.0.0/16)
    Driver: bridge

  • my-api-net (172.21.0.0/16)
    Driver: bridge
```

### omd proxy add

Manually add a reverse proxy rule.

```bash
omd proxy add DOMAIN TARGET
```

**Example:**

```bash
omd proxy add example.local backend:3000
```

### omd proxy remove

Remove a reverse proxy rule.

```bash
omd proxy remove DOMAIN
```

### omd proxy list

List all proxy rules.

```bash
omd proxy list
```

### omd proxy reload

Reload Caddy configuration.

```bash
omd proxy reload
```

### omd ports

Display port mappings across all networks.

```bash
# All networks
omd ports

# Specific network
omd ports show my-api-net
```

## Project Configuration

### omd.toml Structure

The `omd.toml` file is located in your project directory:

```toml
[project]
# Project name (used for container naming)
name = "my-project"

# Domain for this project
domain = "my-project.local"

# Optional: Path to docker-compose file (relative to project directory)
# Defaults to "docker-compose.yml" if not specified
# compose_file = "docker/docker-compose.yml"
# compose_file = "docker-compose.dev.yml"

[network]
# Docker network name for this project
name = "my-project-net"

[caddy]
# Custom Caddy routes (optional)
routes = {}
```

### Configuration Fields

**[project] Section:**

- `name` (required): Project identifier, used in container naming
- `domain` (required): Base domain for accessing services
- `compose_file` (optional): Path to docker-compose file, relative to project directory. Defaults to `"docker-compose.yml"`
- `path` (optional): Automatically filled by `omd up`

**[network] Section:**

- `name` (required): Docker network name for the project

**[caddy] Section:**

- `routes` (optional): Custom route mappings

### Automatic Route Generation

If `[caddy.routes]` is empty or not specified, omd automatically generates routes from your `docker-compose.yml`:

**Example docker-compose.yml:**

```yaml
services:
  frontend:
    image: my-frontend
    ports:
      - "8080:80"
    networks:
      - myapp-net

  backend:
    image: my-backend
    ports:
      - "3000:3000"
    networks:
      - myapp-net
```

**Generated Routes:**

- `frontend.my-project.local` → `frontend-container:80`
- `backend.my-project.local` → `backend-container:3000`

### Custom Routes

Override automatic routing with custom routes:

```toml
[project]
name = "my-project"
domain = "my-project.local"

[network]
name = "my-project-net"

[caddy.routes]
# Custom route: subdomain -> container:port
api = "my-backend-container:3000"
app = "my-frontend-container:80"
admin = "my-admin-panel:8080"
```

**Generated Routes:**

- `api.my-project.local` → `my-backend-container:3000`
- `app.my-project.local` → `my-frontend-container:80`
- `admin.my-project.local` → `my-admin-panel:8080`

### Container Name Detection

omd detects container names in the following order:

1. **Explicit `container_name`** in docker-compose.yml:
   ```yaml
   services:
     api:
       image: my-api
       container_name: my-custom-api
   ```

2. **Generated name** (if no `container_name`):
   - Format: `{project-name}-{service-name}-1`
   - Example: `my-project-api-1`

## Advanced Usage

### Port Conflict Detection

When you run `omd up`, the tool checks the global registry for port conflicts:

**Scenario:**

Project A uses port 5432 for PostgreSQL.

You try to configure Project B, which also uses port 5432.

**Result:**

```
✗ Port conflicts detected:
  Port 5432 is already used by project project-a

Cannot proceed due to port conflicts.
Please update your docker-compose.yml to use different ports.
```

**Solution:**

Update Project B's `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:15
    ports:
      - "5433:5432"  # Changed from 5432:5432
```

### Multiple Projects

You can run multiple projects simultaneously as long as they don't have port conflicts:

```bash
# Project 1
cd ~/projects/api-service
omd init
omd project up
# Access at https://api-service.local

# Project 2
cd ~/projects/web-app
omd init
omd project up
# Access at https://web-app.local
```

### Network Isolation

Each project can have its own network, or projects can share networks:

**Isolated Networks (Default):**

```toml
# Project A
[network]
name = "project-a-net"

# Project B
[network]
name = "project-b-net"
```

**Shared Network:**

```toml
# Both projects
[network]
name = "shared-microservices-net"
```

### SSL Certificates

Place your SSL certificates in `~/.oh-my-dockers/caddy/certs/`:

```
~/.oh-my-dockers/caddy/certs/
├── my_project_local.crt
└── my_project_local.key
```

Certificate naming: Replace `.` with `_` in domain names.

Example: `my-project.local` → `my_project_local.crt`

## Troubleshooting

### Port Already in Use

**Error:**

```
✗ Port conflicts detected:
  Port 5432 is already used by project another-project
```

**Solution:**

Change the host port in your `docker-compose.yml`:

```yaml
ports:
  - "5433:5432"  # Use a different host port
```

### omd.toml Not Found

**Error:**

```
No omd.toml found in current directory. Run 'omd init' to create one.
```

**Solution:**

Run `omd init` in your project directory, or navigate to the correct directory.

### docker-compose.yml Not Found

**Error:**

```
No docker-compose.yml found in current directory.
Please create a docker-compose.yml before running 'omd up'.
```

**Solution:**

Create a `docker-compose.yml` file in your project directory.

### Caddy Not Reloading

**Problem**: Changes to routes not taking effect.

**Solution:**

```bash
omd proxy reload
```

### Project Still Registered After Deletion

**Problem**: Deleted a project directory but it's still in the registry.

**Solution:**

Navigate to a backup copy of `omd.toml` or manually create it, then run:

```bash
omd down
```

Alternatively, edit `~/.oh-my-dockers/registry.json` (not recommended).

### Network Already Exists Error

**Problem**: Docker says network already exists.

**Solution:**

This is usually harmless. omd reuses existing networks. If there's an actual problem:

```bash
# List networks
docker network ls

# Remove if needed (ensure no containers are using it)
docker network rm network-name
```

## Best Practices

### 1. One omd.toml Per Project

Keep `omd.toml` in your project's root directory, alongside `docker-compose.yml`.

### 2. Use Explicit Container Names

For production-like environments, specify container names explicitly:

```yaml
services:
  api:
    image: my-api
    container_name: my-project-api
```

### 3. Document Port Allocations

Add comments to your `docker-compose.yml`:

```yaml
services:
  postgres:
    ports:
      - "5432:5432"  # Standard PostgreSQL port
```

### 4. Consistent Naming

Use consistent naming across project name, network name, and domain:

```toml
[project]
name = "my-awesome-app"
domain = "my-awesome-app.local"

[network]
name = "my-awesome-app-net"
```

### 5. Use omd project up

Use `omd project up` to configure and start your project in one command:

```bash
omd project up   # Configure infrastructure and start containers
```

### 6. Clean Up After Deletion

When deleting a project, use `omd project remove`:

```bash
cd /path/to/project
omd project remove   # Stop containers and remove configuration
cd ..
rm -rf project       # Delete directory
```

### 7. Version Control

Add `omd.toml` to version control, but not the registry:

```gitignore
# .gitignore
# Don't add registry.json to version control
```

But DO commit:
```
# Commit omd.toml
git add omd.toml
```

### 8. Use Port Ranges Wisely

Allocate port ranges for different project types:

- Databases: 5000-5999
- APIs: 3000-3999
- Web servers: 8000-8999

### 9. Network Naming Convention

Use descriptive network names:

- `{project}-net` for isolated projects
- `{service-type}-shared` for shared networks (e.g., `microservices-shared`)

### 10. Regular Registry Review

Periodically review registered projects:

```bash
omd project list
```

Remove stale entries with `omd project remove`.

---

For more information, visit the [GitHub repository](https://github.com/your-repo/oh-my-dockers).
