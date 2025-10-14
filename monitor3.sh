#!/bin/bash
# Real-time streaming build monitor with color-coded output
# Run from R-Com directory

cd /home/markur/R-Com

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     Real-Time Backend Build Monitor                       ║${NC}"
echo -e "${CYAN}║     Press Ctrl+C to exit                                   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to colorize output
colorize_line() {
  local line="$1"

  if echo "$line" | grep -q "Compiling.*backend"; then
    echo -e "${MAGENTA}>>> $line${NC}"
  elif echo "$line" | grep -q "Compiling"; then
    echo -e "${GREEN}$line${NC}"
  elif echo "$line" | grep -q "Finished release"; then
    echo -e "${CYAN}██████ $line ██████${NC}"
  elif echo "$line" | grep -qi "error"; then
    echo -e "${RED}!!! $line !!!${NC}"
  elif echo "$line" | grep -qi "warning"; then
    echo -e "${YELLOW}⚠ $line${NC}"
  elif echo "$line" | grep -q "Downloaded"; then
    echo -e "${BLUE}$line${NC}"
  elif echo "$line" | grep -q "Backend running"; then
    echo -e "${GREEN}✓✓✓ $line ✓✓✓${NC}"
  elif echo "$line" | grep -q "Listening on"; then
    echo -e "${GREEN}✓✓✓ $line ✓✓✓${NC}"
  else
    echo "$line"
  fi
}

# Stream logs with colorization
docker compose logs -f --tail=50 backend 2>&1 | while IFS= read -r line; do
  colorize_line "$line"
done
