#!/bin/bash
# Comprehensive test script for oh-my-dockers CLI tool

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

TEST_DIR="/tmp/oh-my-dockers-test-$$"
export OH_MY_DOCKERS_DIR="$TEST_DIR"

echo "🧪 Starting comprehensive tests for oh-my-dockers"
echo "📁 Test directory: $TEST_DIR"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up test directory..."
    rm -rf "$TEST_DIR"
    echo "✅ Cleanup complete"
}

trap cleanup EXIT

# Helper function to run commands
run_cmd() {
    echo "▶️  Running: cargo run --bin oh-my-dockers -- $*"
    (cd "$PROJECT_ROOT" && cargo run --bin oh-my-dockers -- "$@" 2>&1 | grep -v "warning:" | grep -v "Finished" | grep -v "Compiling" | grep -v "Running") || true
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Configuration Directory Auto-Creation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
run_cmd project list

if [ -d "$TEST_DIR" ]; then
    echo "✅ Config directory created successfully"
    echo "Directory structure:"
    find "$TEST_DIR" -type d | sort
    echo ""
    if [ -f "$TEST_DIR/config.toml" ]; then
        echo "✅ Default config.toml created"
        echo "Config file preview:"
        head -5 "$TEST_DIR/config.toml"
        echo ""
    fi
else
    echo "❌ Config directory was not created"
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Network Management"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2.1: List networks"
run_cmd network list

echo "2.2: Create test network"
run_cmd network create "test-network-$$"

echo "2.3: Verify network exists"
run_cmd network list | grep -q "test-network-$$" && echo "✅ Network created successfully" || echo "⚠️  Network may not be visible"

echo "2.4: Remove test network"
run_cmd network remove "test-network-$$"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: Proxy Management"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3.1: List proxy rules"
run_cmd proxy list

echo "3.2: Add a test proxy rule"
run_cmd proxy add "test-$$.local" "test-container:8080"

if [ -f "$TEST_DIR/caddy/projects/test___local.caddy" ]; then
    echo "✅ Proxy rule file created"
    echo "Proxy config content:"
    cat "$TEST_DIR/caddy/projects/test___local.caddy"
    echo ""
fi

echo "3.3: List proxy rules again"
run_cmd proxy list

echo "3.4: Remove test proxy rule"
run_cmd proxy remove "test-$$.local"

echo "3.5: Verify removal"
run_cmd proxy list
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Port Mapping Display"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4.1: List all port mappings"
run_cmd ports

echo "4.2: Show ports for specific network (if exists)"
if docker network ls --format "{{.Name}}" | grep -q "^bridge$"; then
    run_cmd ports show bridge
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 5: Project Management"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5.1: List projects"
run_cmd project list

echo "5.2: Create a test project config"
mkdir -p "$TEST_DIR/projects"
cat > "$TEST_DIR/projects/test-project.toml" << 'EOF'
[project]
name = "test-project"
domain = "test-project.local"
mode = "managed"

[network]
name = "test-project-net"

[caddy]
auto_subdomains = false
EOF

echo "5.3: List projects again"
run_cmd project list
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 6: Migration Tool"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create test data in a temporary directory
TEST_SOURCE_DIR="/tmp/oh-my-dockers-migrate-test-$$"
mkdir -p "$TEST_SOURCE_DIR"/projects
mkdir -p "$TEST_SOURCE_DIR"/templates
mkdir -p "$TEST_SOURCE_DIR"/caddy/projects
mkdir -p "$TEST_SOURCE_DIR"/caddy/certs

# Create a test project config
cat > "$TEST_SOURCE_DIR/projects/test-migrate.toml" << 'EOF'
[project]
name = "test-migrate"
domain = "test-migrate.local"
mode = "managed"

[network]
name = "test-migrate-net"

[caddy]
auto_subdomains = true
EOF

# Create a test template
cat > "$TEST_SOURCE_DIR/templates/postgres.yml" << 'EOF'
services:
  postgres:
    image: postgres:${POSTGRES_VERSION:-latest}
EOF

# Create a test Caddy config
cat > "$TEST_SOURCE_DIR/caddy/projects/test-migrate.caddy" << 'EOF'
test-migrate.local {
    reverse_proxy test-migrate:80
}
EOF

# Create a test cert file
touch "$TEST_SOURCE_DIR/caddy/certs/test.crt"

# Create a test config.toml
cat > "$TEST_SOURCE_DIR/config.toml" << 'EOF'
[global]
caddy_network = "caddy-net"
projects_dir = "./projects"
templates_dir = "./templates"
EOF

echo "Test source directory structure:"
find "$TEST_SOURCE_DIR" -type f | head -10
echo ""

# Change to test source directory and run migration
cd "$TEST_SOURCE_DIR"
export OH_MY_DOCKERS_DIR="$TEST_DIR"
(cd "$PROJECT_ROOT" && cargo run --bin oh-my-dockers -- migrate 2>&1 | grep -v "warning:" | grep -v "Finished" | grep -v "Compiling" | grep -v "Running") || true
cd "$PROJECT_ROOT"

# Verify migration
MIGRATION_SUCCESS=true
if [ -f "$TEST_DIR/projects/test-migrate.toml" ]; then
    echo "✅ Project config migrated successfully"
else
    echo "❌ Project config not migrated"
    MIGRATION_SUCCESS=false
fi

if [ -f "$TEST_DIR/templates/postgres.yml" ]; then
    echo "✅ Template migrated successfully"
else
    echo "❌ Template not migrated"
    MIGRATION_SUCCESS=false
fi

if [ -f "$TEST_DIR/caddy/projects/test-migrate.caddy" ]; then
    echo "✅ Caddy config migrated successfully"
else
    echo "❌ Caddy config not migrated"
    MIGRATION_SUCCESS=false
fi

if [ -f "$TEST_DIR/caddy/certs/test.crt" ]; then
    echo "✅ Certificates migrated successfully"
else
    echo "⚠️  Certificates not migrated (may be expected)"
fi

# Cleanup test source
rm -rf "$TEST_SOURCE_DIR"

if [ "$MIGRATION_SUCCESS" = true ]; then
    echo "✅ Migration test passed"
else
    echo "❌ Migration test failed"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 7: Help Commands Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7.1: Main help"
run_cmd --help | head -15

echo "7.2: Network help"
run_cmd network --help | head -10

echo "7.3: Proxy help"
run_cmd proxy --help | head -10

echo "7.4: Ports help"
run_cmd ports --help

echo "7.5: Project help"
run_cmd project --help | head -10
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 8: Environment Variable Support"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
CUSTOM_DIR="/tmp/custom-oh-my-dockers-$$"
export OH_MY_DOCKERS_DIR="$CUSTOM_DIR"
run_cmd project list

if [ -d "$CUSTOM_DIR" ]; then
    echo "✅ Custom config directory created via OH_MY_DOCKERS_DIR"
    rm -rf "$CUSTOM_DIR"
else
    echo "❌ Custom config directory not created"
fi

export OH_MY_DOCKERS_DIR="$TEST_DIR"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All tests completed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
