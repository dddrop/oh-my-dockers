#!/bin/bash
# Test script for compose_file configuration option

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║   Testing compose_file Configuration Feature         ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

OMD_BIN="./target/release/omd"

# Test 1: Compose file in subdirectory
echo "=== Test 1: Compose file in subdirectory ==="
cd /tmp && rm -rf test-compose-subdir && mkdir -p test-compose-subdir/docker
cd test-compose-subdir

cat > docker/docker-compose.yml <<'EOF'
services:
  api:
    image: nginx:alpine
    container_name: test-subdir-api
    ports:
      - "9001:80"
    networks:
      - test-subdir-net

networks:
  test-subdir-net:
EOF

cat > omd.toml <<'EOF'
[project]
name = "test-subdir"
domain = "test-subdir.local"
compose_file = "docker/docker-compose.yml"

[network]
name = "test-subdir-net"
EOF

echo "✓ Created project with compose_file = \"docker/docker-compose.yml\""
echo "Running: omd project up"
$OMD_BIN project up 2>&1 | head -20
echo ""

# Test 2: Different compose file name
echo "=== Test 2: Different compose file name ==="
cd /tmp && rm -rf test-compose-name && mkdir test-compose-name
cd test-compose-name

cat > docker-compose.dev.yml <<'EOF'
services:
  app:
    image: nginx:alpine
    container_name: test-name-app
    ports:
      - "9002:80"
    networks:
      - test-name-net

networks:
  test-name-net:
EOF

cat > omd.toml <<'EOF'
[project]
name = "test-name"
domain = "test-name.local"
compose_file = "docker-compose.dev.yml"

[network]
name = "test-name-net"
EOF

echo "✓ Created project with compose_file = \"docker-compose.dev.yml\""
echo "Running: omd project up"
$OMD_BIN project up 2>&1 | head -20
echo ""

# Test 3: Default behavior (no compose_file specified)
echo "=== Test 3: Default behavior ==="
cd /tmp && rm -rf test-compose-default && mkdir test-compose-default
cd test-compose-default

cat > docker-compose.yml <<'EOF'
services:
  web:
    image: nginx:alpine
    container_name: test-default-web
    ports:
      - "9003:80"
    networks:
      - test-default-net

networks:
  test-default-net:
EOF

cat > omd.toml <<'EOF'
[project]
name = "test-default"
domain = "test-default.local"

[network]
name = "test-default-net"
EOF

echo "✓ Created project without compose_file (should default to docker-compose.yml)"
echo "Running: omd project up"
$OMD_BIN project up 2>&1 | head -20
echo ""

# Cleanup
echo "=== Cleanup ==="
cd /tmp/test-compose-subdir && $OMD_BIN project down 2>&1 | head -5 || true
cd /tmp/test-compose-name && $OMD_BIN project down 2>&1 | head -5 || true
cd /tmp/test-compose-default && $OMD_BIN project down 2>&1 | head -5 || true
rm -rf /tmp/test-compose-{subdir,name,default}
docker network rm test-subdir-net test-name-net test-default-net 2>/dev/null || true
echo "✓ Cleanup complete"

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║          All compose_file Tests Completed            ║"
echo "╚════════════════════════════════════════════════════════╝"

