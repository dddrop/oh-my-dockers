# PostgreSQL Initialization Scripts

This directory contains SQL scripts that are automatically executed when the PostgreSQL container is first initialized.

## Files

- `create_n8n.sql` - Creates the n8n user and schema for n8n workflow automation

## Usage

These scripts are automatically executed by PostgreSQL when the container is first created. They are mounted to `/docker-entrypoint-initdb.d/` in the container.

Scripts are executed in alphabetical order.

## Adding Custom Initialization Scripts

You can add your own `.sql` or `.sh` scripts to this directory. They will be executed when the database is first initialized.

Example:
```sql
-- custom_init.sql
CREATE DATABASE myapp;
CREATE USER myapp_user WITH PASSWORD 'password';
GRANT ALL PRIVILEGES ON DATABASE myapp TO myapp_user;
```

## Notes

- Scripts are **only** executed on first initialization
- To re-run scripts, you need to delete the volume: `docker volume rm daily_postgres_data`
- Scripts run as the postgres superuser

