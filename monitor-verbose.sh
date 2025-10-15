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
DIM='\033[2m'
NC='\033[0m'

# Progress tracking
COMPILE_COUNT=0
TOTAL_CRATES=0
START_TIME=$(date +%s)

echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║   Verbose Backend Build Monitor - Real-Time Compilation   ║${NC}"
echo -e "${BOLD}${CYAN}║   Press Ctrl+C to exit                                     ║${NC}"
echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Monitoring: ${NC}docker compose build backend (live)"
echo -e "${BLUE}Started: ${NC}$(date)"
echo ""

# Function to show elapsed time
show_elapsed() {
  local current=$(date +%s)
  local elapsed=$((current - START_TIME))
  local mins=$((elapsed / 60))
  local secs=$((elapsed % 60))
  echo -e "${DIM}[${mins}m ${secs}s]${NC}"
}

# Function to show progress bar
show_progress() {
  if [ $TOTAL_CRATES -gt 0 ]; then
    local percent=$((COMPILE_COUNT * 100 / TOTAL_CRATES))
    local filled=$((percent / 5))
    local empty=$((20 - filled))
    printf "${CYAN}["
    printf "%${filled}s" | tr ' ' '='
    printf "%${empty}s" | tr ' ' '-'
    printf "] ${percent}%% (${COMPILE_COUNT}/${TOTAL_CRATES})${NC} "
  fi
}

# Function to colorize compilation output
colorize_build() {
  local line="$1"

  # Track total crates from Locking message
  if echo "$line" | grep -q "Locking.*packages"; then
    TOTAL_CRATES=$(echo "$line" | grep -oP '\d+(?= packages)')
    echo -e "${MAGENTA}📦 Locking $TOTAL_CRATES packages...${NC}"

  # Highlight Compiling lines
  elif echo "$line" | grep -q "Compiling.*backend v"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    show_progress
    echo -e "${BOLD}${MAGENTA}>>> COMPILING MAIN BACKEND${NC} $(show_elapsed)"

  elif echo "$line" | grep -q "Compiling"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    # Extract package name and version
    PKG=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\) \(v[^ ]*\).*/\1 \2/p')
    if [ -n "$PKG" ]; then
      show_progress
      echo -e "${GREEN}⚙  $PKG${NC}"
    else
      echo -e "${GREEN}$line${NC}"
    fi

  # Build stages
  elif echo "$line" | grep -q "Finished.*release"; then
    local elapsed=$(($(date +%s) - START_TIME))
    echo ""
    echo -e "${BOLD}${CYAN}══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${GREEN}✓✓✓ BUILD SUCCESSFUL ✓✓✓${NC}"
    echo -e "${CYAN}Time: ${elapsed}s | Crates: ${COMPILE_COUNT}${NC}"
    echo -e "${BOLD}${CYAN}══════════════════════════════════════════════════════${NC}"

  elif echo "$line" | grep -q "Finished"; then
    echo -e "${CYAN}✓ $line${NC}"

  # Docker build stages
  elif echo "$line" | grep -q "^\[builder"; then
    echo -e "${BLUE}$line${NC}"

  elif echo "$line" | grep -q "^\[stage-"; then
    echo -e "${BLUE}$line${NC}"

  # Errors with context
  elif echo "$line" | grep -qi "error\[E[0-9]*\]"; then
    echo ""
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                    COMPILATION ERROR                      ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${BOLD}${RED}$line${NC}"

  elif echo "$line" | grep -qi "^error:"; then
    echo -e "${BOLD}${RED}!!! $line !!!${NC}"

  elif echo "$line" | grep -qi "error"; then
    echo -e "${RED}$line${NC}"

  # Warnings in YELLOW
  elif echo "$line" | grep -qi "warning\["; then
    echo -e "${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "^warning:"; then
    echo -e "${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "warning"; then
    echo -e "${YELLOW}$line${NC}"

  # Help messages
  elif echo "$line" | grep -q "help:"; then
    echo -e "${CYAN}💡 $line${NC}"

  # Note messages
  elif echo "$line" | grep -q "note:"; then
    echo -e "${DIM}📝 $line${NC}"

  # Arrow indicators (error context)
  elif echo "$line" | grep -q "\-\->"; then
    echo -e "${YELLOW}$line${NC}"

  elif echo "$line" | grep -qE '^\s*\|'; then
    echo -e "${DIM}$line${NC}"

  elif echo "$line" | grep -qE '^\s*\^'; then
    echo -e "${RED}$line${NC}"

  # Downloading/fetching
  elif echo "$line" | grep -q "Downloaded\|Downloading"; then
    echo -e "${DIM}${BLUE}⬇  $line${NC}"

  elif echo "$line" | grep -q "Fetching\|Updating crates.io"; then
    echo -e "${BLUE}🔄 $line${NC}"

  # Lock/Update messages
  elif echo "$line" | grep -q "Adding\|Updating.*->"; then
    echo -e "${DIM}${MAGENTA}$line${NC}"

  elif echo "$line" | grep -q "Downgrading"; then
    echo -e "${YELLOW}⬇  $line${NC}"

  # Backend running messages
  elif echo "$line" | grep -q "Backend running\|Listening on"; then
    echo ""
    echo -e "${BOLD}${GREEN}✓✓✓ $line ✓✓✓${NC}"
    echo ""

  # Build success indicator
  elif echo "$line" | grep -q "Successfully built\|Successfully tagged"; then
    echo -e "${BOLD}${GREEN}✓ $line${NC}"

  # Failed to solve
  elif echo "$line" | grep -qi "failed to solve"; then
    echo ""
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                      BUILD FAILED                         ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${BOLD}${RED}$line${NC}"

  # Show certain patterns dimmed
  elif echo "$line" | grep -qE '^\s*$|^#[0-9]+ DONE|^#[0-9]+ CACHED'; then
    echo -e "${DIM}$line${NC}"

  # Default - show as-is with slight dim for less important lines
  else
    # Check if it's a continuation of error context
    if echo "$line" | grep -qE '^\s+[0-9]+\s*\|'; then
      echo -e "${DIM}$line${NC}"
    else
      echo "$line"
    fi
  fi
}

# Check if we're monitoring build or logs
if docker compose ps backend 2>/dev/null | grep -q "Up"; then
  MODE="logs"
  echo -e "${YELLOW}ℹ  Backend container is running - monitoring logs${NC}"
  echo -e "${YELLOW}ℹ  To monitor a build, run: docker compose build backend${NC}"
  echo ""
  docker compose logs -f --tail=100 backend 2>&1 | while IFS= read -r line; do
    colorize_build "$line"
  done
else
  MODE="build"
  echo -e "${YELLOW}ℹ  Backend container not running - use this to monitor builds${NC}"
  echo -e "${YELLOW}ℹ  Run in another terminal: docker compose build backend${NC}"
  echo -e "${YELLOW}ℹ  Or press Ctrl+C and run: docker compose build backend 2>&1 | ./monitor-verbose.sh${NC}"
  echo ""

  # Read from stdin if piped, otherwise tail docker logs
  if [ -p /dev/stdin ]; then
    while IFS= read -r line; do
      colorize_build "$line"
    done
  else
    echo -e "${RED}⚠  No input piped. Usage: docker compose build backend 2>&1 | ./monitor-verbose.sh${NC}"
    exit 1
  fi
fi
