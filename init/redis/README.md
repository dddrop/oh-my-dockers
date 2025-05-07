# Redis Configuration

This directory contains configuration files for Redis.

## Files

- `redis.conf` - Redis server configuration

## Configuration Highlights

- **Persistence**: Both RDB snapshots and AOF (Append Only File) enabled
- **Memory**: 256MB limit with LRU eviction policy
- **Network**: Listening on all interfaces (safe for Docker networks)

## Customization

You can modify `redis.conf` to adjust:
- Memory limits (`maxmemory`)
- Persistence settings (`save`, `appendonly`)
- Eviction policy (`maxmemory-policy`)

## Usage

This configuration is mounted to the Redis container and used when Redis starts.

## Security Note

For local development, authentication is handled via the `REDIS_PASSWORD` environment variable set in your project's `.env` file.

