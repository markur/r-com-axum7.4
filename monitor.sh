#!/bin/bash
# Backend monitoring script
# Run from R-Com directory

cd /home/markur/R-Com

while true; do
  clear
  echo "╔════════════════════════════════════════════════════════════╗"
  echo "║          Backend Build & Runtime Monitor                  ║"
  echo "╚════════════════════════════════════════════════════════════╝"
  echo ""

  echo "=== Container Status ==="
  docker compose ps backend 2>/dev/null || echo "Container not found or docker-compose.yml missing"
  echo ""

  echo "=== Health Check ==="
  if curl -s --connect-timeout 1 http://localhost:3000/ >/dev/null 2>&1; then
    echo "✓ Backend is RESPONDING (http://localhost:3000/)"
  else
    echo "✗ Backend not ready (building or starting...)"
  fi
  echo ""

  echo "=== Build Progress ==="
  # Show compilation status if available
  docker compose logs --tail=1 backend 2>/dev/null | grep -E "(Compiling|Finished|Building|error)" || echo "No recent build activity"
  echo ""

  echo "=== Latest Log Lines ==="
  docker compose logs --tail=5 backend 2>/dev/null || echo "No logs available"
  echo ""

  echo "Press Ctrl+C to exit"
  echo "Refreshing in 3 seconds..."
  sleep 3
done
