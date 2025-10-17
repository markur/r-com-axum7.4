#!/bin/bash
# Multi-Service Build Monitor - Track backend, frontend-leptos, and other services
# Shows real-time compilation progress for all builds
# Run from R-Com directory

cd /home/markur/R-Com

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
PURPLE='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Progress tracking per service
declare -A COMPILE_COUNT
declare -A TOTAL_CRATES
declare -A START_TIME
declare -A SERVICE_COLOR

# Initialize services
SERVICES=("backend" "frontend-leptos")
SERVICE_COLOR["backend"]="${BLUE}"
SERVICE_COLOR["frontend-leptos"]="${MAGENTA}"

for service in "${SERVICES[@]}"; do
  COMPILE_COUNT[$service]=0
  TOTAL_CRATES[$service]=0
  START_TIME[$service]=$(date +%s)
done

GLOBAL_START=$(date +%s)

echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║     Multi-Service Build Monitor - Real-Time Tracking      ║${NC}"
echo -e "${BOLD}${CYAN}║     Press Ctrl+C to exit                                   ║${NC}"
echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Services: ${NC}backend, frontend-leptos"
echo -e "${BLUE}Started: ${NC}$(date)"
echo ""

# Function to show elapsed time for a service
show_elapsed() {
  local service="$1"
  local current=$(date +%s)
  local start=${START_TIME[$service]}
  local elapsed=$((current - start))
  local mins=$((elapsed / 60))
  local secs=$((elapsed % 60))
  printf "${DIM}[${mins}m ${secs}s]${NC}"
}

# Function to show progress bar for a service
show_progress() {
  local service="$1"
  local count=${COMPILE_COUNT[$service]}
  local total=${TOTAL_CRATES[$service]}
  local color="${SERVICE_COLOR[$service]}"

  if [ $total -gt 0 ]; then
    local percent=$((count * 100 / total))
    local filled=$((percent / 5))
    local empty=$((20 - filled))
    printf "${color}["
    printf "%${filled}s" | tr ' ' '='
    printf "%${empty}s" | tr ' ' '-'
    printf "] ${percent}%% (${count}/${total})${NC} "
  fi
}

# Function to detect which service a line belongs to
detect_service() {
  local line="$1"

  # Check for Leptos/WASM/trunk specific patterns
  if echo "$line" | grep -qE "frontend-leptos|trunk|wasm32|leptos|WASM"; then
    echo "frontend-leptos"
  # Check for backend specific patterns
  elif echo "$line" | grep -qE "backend|axum|sqlx|tokio"; then
    echo "backend"
  # Default to backend for Rust compilation
  elif echo "$line" | grep -q "Compiling"; then
    echo "backend"
  else
    echo "unknown"
  fi
}

# Function to colorize compilation output
colorize_build() {
  local line="$1"
  local service=$(detect_service "$line")
  local color="${SERVICE_COLOR[$service]:-${NC}}"
  local prefix=""

  # Add service prefix for clarity
  if [ "$service" != "unknown" ]; then
    prefix="${BOLD}${color}[${service}]${NC} "
  fi

  # Track total crates from Locking message
  if echo "$line" | grep -q "Locking.*packages"; then
    local count=$(echo "$line" | grep -oP '\d+(?= packages)')
    if [ "$service" != "unknown" ]; then
      TOTAL_CRATES[$service]=$count
    fi
    echo -e "${prefix}${MAGENTA}📦 Locking $count packages...${NC}"

  # Highlight Compiling lines
  elif echo "$line" | grep -q "Compiling.*backend v"; then
    COMPILE_COUNT["backend"]=$((COMPILE_COUNT["backend"] + 1))
    show_progress "backend"
    echo -e "${prefix}${BOLD}${MAGENTA}>>> COMPILING MAIN BACKEND${NC} $(show_elapsed backend)"

  elif echo "$line" | grep -q "Compiling.*frontend-leptos"; then
    COMPILE_COUNT["frontend-leptos"]=$((COMPILE_COUNT["frontend-leptos"] + 1))
    show_progress "frontend-leptos"
    echo -e "${prefix}${BOLD}${PURPLE}>>> COMPILING LEPTOS FRONTEND${NC} $(show_elapsed frontend-leptos)"

  elif echo "$line" | grep -q "Compiling"; then
    if [ "$service" != "unknown" ]; then
      COMPILE_COUNT[$service]=$((COMPILE_COUNT[$service] + 1))
    fi
    # Extract package name and version
    PKG=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\) \(v[^ ]*\).*/\1 \2/p')
    if [ -n "$PKG" ]; then
      if [ "$service" != "unknown" ]; then
        show_progress "$service"
      fi
      echo -e "${prefix}${GREEN}⚙  $PKG${NC}"
    else
      echo -e "${prefix}${GREEN}$line${NC}"
    fi

  # Trunk-specific build stages
  elif echo "$line" | grep -q "Finished.*trunk"; then
    echo -e "${prefix}${BOLD}${GREEN}✓ Trunk build completed${NC}"

  elif echo "$line" | grep -q "Building.*trunk"; then
    echo -e "${prefix}${CYAN}🔨 Building with trunk...${NC}"

  # WASM optimization
  elif echo "$line" | grep -qi "wasm-opt\|optimizing.*wasm"; then
    echo -e "${prefix}${CYAN}⚡ Optimizing WASM bundle...${NC}"

  # Build stages
  elif echo "$line" | grep -q "Finished.*release"; then
    if [ "$service" != "unknown" ]; then
      local elapsed=$(($(date +%s) - START_TIME[$service]))
      echo ""
      echo -e "${BOLD}${color}══════════════════════════════════════════════════════${NC}"
      echo -e "${prefix}${BOLD}${GREEN}✓✓✓ BUILD SUCCESSFUL ✓✓✓${NC}"
      echo -e "${prefix}${CYAN}Time: ${elapsed}s | Crates: ${COMPILE_COUNT[$service]}${NC}"
      echo -e "${BOLD}${color}══════════════════════════════════════════════════════${NC}"
    fi

  elif echo "$line" | grep -q "Finished"; then
    echo -e "${prefix}${CYAN}✓ $line${NC}"

  # Docker build stages
  elif echo "$line" | grep -q "^\[builder"; then
    echo -e "${prefix}${BLUE}$line${NC}"

  elif echo "$line" | grep -q "^\[stage-"; then
    echo -e "${prefix}${BLUE}$line${NC}"

  # Errors with context
  elif echo "$line" | grep -qi "error\[E[0-9]*\]"; then
    echo ""
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                    COMPILATION ERROR                      ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${prefix}${BOLD}${RED}$line${NC}"

  elif echo "$line" | grep -qi "^error:"; then
    echo -e "${prefix}${BOLD}${RED}!!! $line !!!${NC}"

  elif echo "$line" | grep -qi "error"; then
    echo -e "${prefix}${RED}$line${NC}"

  # Warnings in YELLOW
  elif echo "$line" | grep -qi "warning\["; then
    echo -e "${prefix}${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "^warning:"; then
    echo -e "${prefix}${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "warning"; then
    echo -e "${prefix}${YELLOW}$line${NC}"

  # Help messages
  elif echo "$line" | grep -q "help:"; then
    echo -e "${prefix}${CYAN}💡 $line${NC}"

  # Downloading/fetching
  elif echo "$line" | grep -q "Downloaded\|Downloading"; then
    echo -e "${prefix}${DIM}${BLUE}⬇  $line${NC}"

  elif echo "$line" | grep -q "Fetching\|Updating crates.io"; then
    echo -e "${prefix}${BLUE}🔄 $line${NC}"

  # Build success indicator
  elif echo "$line" | grep -q "Successfully built\|Successfully tagged"; then
    echo -e "${prefix}${BOLD}${GREEN}✓ $line${NC}"

  # Failed to solve
  elif echo "$line" | grep -qi "failed to solve"; then
    echo ""
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                      BUILD FAILED                         ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${prefix}${BOLD}${RED}$line${NC}"

  # Show certain patterns dimmed
  elif echo "$line" | grep -qE '^\s*$|^#[0-9]+ DONE|^#[0-9]+ CACHED'; then
    echo -e "${DIM}$line${NC}"

  # Default
  else
    if echo "$line" | grep -qE '^\s+[0-9]+\s*\|'; then
      echo -e "${DIM}$line${NC}"
    else
      echo "$line"
    fi
  fi
}

# Read from stdin if piped
if [ -p /dev/stdin ]; then
  while IFS= read -r line; do
    colorize_build "$line"
  done
else
  echo -e "${YELLOW}Usage:${NC}"
  echo -e "  ${CYAN}# Monitor single service:${NC}"
  echo -e "  docker compose build backend 2>&1 | ./monitor-all-builds.sh"
  echo -e "  docker compose build frontend-leptos 2>&1 | ./monitor-all-builds.sh"
  echo ""
  echo -e "  ${CYAN}# Monitor all builds (run in separate terminals):${NC}"
  echo -e "  docker compose build backend 2>&1 | tee /tmp/backend-build.log &"
  echo -e "  docker compose build frontend-leptos 2>&1 | tee /tmp/leptos-build.log &"
  echo -e "  tail -f /tmp/*-build.log | ./monitor-all-builds.sh"
  exit 1
fi

# Show summary on exit
trap 'echo ""; echo -e "${BOLD}${CYAN}Build monitoring stopped${NC}"' EXIT
