# Example Configuration

This directory contains example configurations for omd projects.

## Files

- **`omd.toml`** - Example project configuration file
- **`docker-compose.yml`** - Example Docker Compose file

## Quick Start

1. Create a new project directory:
   ```bash
   mkdir my-awesome-project
   cd my-awesome-project
   ```

2. Initialize omd configuration:
   ```bash
   omd init
   ```
   
   Or copy the example:
   ```bash
   cp /path/to/oh-my-dockers/examples/omd.toml .
   ```

3. Create your `docker-compose.yml`:
   ```bash
   cp /path/to/oh-my-dockers/examples/docker-compose.yml .
   # Edit as needed
   ```

4. Configure infrastructure:
   ```bash
   omd up
   ```

5. Start services:
   ```bash
   docker compose up -d
   ```

## Configuration Options

### Project Section

```toml
[project]
name = "my-project"      # Project identifier
domain = "my-project.local"  # Base domain
```

### Network Section

```toml
[network]
name = "my-project-net"  # Docker network name
```

### Caddy Routes (Optional)

```toml
[caddy.routes]
api = "backend:3000"     # api.my-project.local -> backend:3000
app = "frontend:80"      # app.my-project.local -> frontend:80
```

If routes are not specified, omd automatically generates them from your `docker-compose.yml`.

## Tips

1. **Consistent Naming**: Use the same name prefix for project, network, and containers
2. **Port Planning**: Avoid common ports to prevent conflicts (e.g., use 5433 instead of 5432 if needed)
3. **Container Names**: Explicitly set container names for better control
4. **Network Setup**: Ensure all services use the same network name

## More Examples

### Simple API Project

```toml
[project]
name = "api-service"
domain = "api-service.local"

[network]
name = "api-service-net"
```

### Microservices Project with Custom Routes

```toml
[project]
name = "shop"
domain = "shop.local"

[network]
name = "shop-net"

[caddy.routes]
api = "shop-api:3000"
web = "shop-frontend:80"
admin = "shop-admin:8080"
```

## See Also

- [Main README](../README.md)
- [Manual](../docs/MANUAL.md)
- [Manual (日本語)](../docs/MANUAL.ja.md)
- [Manual (中文)](../docs/MANUAL.zh.md)

