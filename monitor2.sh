#!/bin/bash
# Enhanced Backend Monitoring Script with Verbose Build Output
# Run from R-Com directory

cd /home/markur/R-Com

# Colors for better visibility
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

while true; do
  clear
  echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║     Backend Build Monitor - Verbose Mode                  ║${NC}"
  echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
  echo ""

  # Container Status
  echo -e "${BLUE}=== Container Status ===${NC}"
  CONTAINER_STATUS=$(docker compose ps backend 2>/dev/null | tail -n +2)
  if [ -z "$CONTAINER_STATUS" ]; then
    echo -e "${YELLOW}⚠ Container not found or not running${NC}"
  else
    echo "$CONTAINER_STATUS"
  fi
  echo ""

  # Health Check
  echo -e "${BLUE}=== Health Check ===${NC}"
  if curl -s --connect-timeout 1 http://localhost:3000/ >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Backend is LIVE at http://localhost:3000/${NC}"
  else
    echo -e "${YELLOW}✗ Backend not responding (building/starting...)${NC}"
  fi
  echo ""

  # Build Progress - Show last compilation
  echo -e "${BLUE}=== Current Build Activity ===${NC}"
  LAST_COMPILE=$(docker compose logs --tail=50 backend 2>/dev/null | grep "Compiling" | tail -1)
  if [ -n "$LAST_COMPILE" ]; then
    echo -e "${GREEN}$LAST_COMPILE${NC}"
  else
    echo -e "${YELLOW}No active compilation detected${NC}"
  fi
  echo ""

  # Check for errors
  ERRORS=$(docker compose logs --tail=20 backend 2>/dev/null | grep -i "error\|failed" | tail -3)
  if [ -n "$ERRORS" ]; then
    echo -e "${RED}=== Recent Errors ===${NC}"
    echo -e "${RED}$ERRORS${NC}"
    echo ""
  fi

  # Verbose log output (last 25 lines)
  echo -e "${BLUE}=== Verbose Build Log (Last 25 Lines) ===${NC}"
  echo -e "${CYAN}────────────────────────────────────────────────────────────${NC}"
  docker compose logs --tail=25 backend 2>/dev/null | while IFS= read -r line; do
    # Highlight different types of messages
    if echo "$line" | grep -q "Compiling"; then
      echo -e "${GREEN}$line${NC}"
    elif echo "$line" | grep -qi "error"; then
      echo -e "${RED}$line${NC}"
    elif echo "$line" | grep -qi "warning"; then
      echo -e "${YELLOW}$line${NC}"
    elif echo "$line" | grep -q "Finished"; then
      echo -e "${CYAN}$line${NC}"
    else
      echo "$line"
    fi
  done
  echo -e "${CYAN}────────────────────────────────────────────────────────────${NC}"
  echo ""

  # Stats
  echo -e "${BLUE}=== Statistics ===${NC}"
  TOTAL_LINES=$(docker compose logs backend 2>/dev/null | wc -l)
  COMPILE_COUNT=$(docker compose logs backend 2>/dev/null | grep -c "Compiling")
  echo "Total log lines: $TOTAL_LINES"
  echo "Packages compiled: $COMPILE_COUNT"
  echo ""

  echo -e "${CYAN}Press Ctrl+C to exit | Refreshing in 3 seconds...${NC}"
  sleep 3
done
