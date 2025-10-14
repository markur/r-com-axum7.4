#!/bin/bash
# Verbose Backend Build Monitor - Real-time detailed compilation tracking
# Shows every crate being compiled with detailed colorization
# Run from R-Com directory

cd /home/markur/R-Com

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║   Verbose Backend Build Monitor - Real-Time Compilation   ║${NC}"
echo -e "${BOLD}${CYAN}║   Press Ctrl+C to exit                                     ║${NC}"
echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Monitoring: ${NC}docker compose logs -f backend"
echo -e "${BLUE}Started: ${NC}$(date)"
echo ""

# Function to colorize compilation output
colorize_build() {
  local line="$1"

  # Highlight Compiling lines
  if echo "$line" | grep -q "Compiling.*backend"; then
    echo -e "${BOLD}${MAGENTA}>>> $line${NC}"
  elif echo "$line" | grep -q "Compiling"; then
    # Extract package name and version for detailed display
    PKG=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\) \(v[^ ]*\).*/\1 \2/p')
    if [ -n "$PKG" ]; then
      echo -e "${GREEN}  ⚙  Compiling: ${NC}${PKG}"
    else
      echo -e "${GREEN}$line${NC}"
    fi

  # Highlight build stages
  elif echo "$line" | grep -q "Finished.*release"; then
    echo -e "${BOLD}${CYAN}██████ $line ██████${NC}"
    echo -e "${BOLD}${GREEN}✓✓✓ BUILD SUCCESSFUL ✓✓✓${NC}"

  elif echo "$line" | grep -q "Finished"; then
    echo -e "${CYAN}$line${NC}"

  # Errors in RED with emphasis
  elif echo "$line" | grep -qi "error\["; then
    echo -e "${BOLD}${RED}!!! ERROR: $line !!!${NC}"
  elif echo "$line" | grep -qi "^error:"; then
    echo -e "${BOLD}${RED}!!! $line !!!${NC}"
  elif echo "$line" | grep -qi "error"; then
    echo -e "${RED}$line${NC}"

  # Warnings in YELLOW
  elif echo "$line" | grep -qi "warning"; then
    echo -e "${YELLOW}⚠  $line${NC}"

  # Downloading/fetching in BLUE
  elif echo "$line" | grep -q "Downloaded\|Downloading\|Fetching"; then
    echo -e "${BLUE}$line${NC}"

  # Lock/Update messages
  elif echo "$line" | grep -q "Locking\|Updating"; then
    echo -e "${MAGENTA}$line${NC}"

  # Backend running messages
  elif echo "$line" | grep -q "Backend running\|Listening on"; then
    echo -e "${BOLD}${GREEN}✓✓✓ $line ✓✓✓${NC}"

  # Build progress indicators
  elif echo "$line" | grep -q "\[.*\]"; then
    echo -e "${CYAN}$line${NC}"

  # Default - show as-is
  else
    echo "$line"
  fi
}

# Stream logs with colorization
docker compose logs -f --tail=100 backend 2>&1 | while IFS= read -r line; do
  colorize_build "$line"
done
