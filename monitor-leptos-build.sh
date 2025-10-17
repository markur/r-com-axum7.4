#!/bin/bash
# Leptos Frontend Build Monitor - Real-time WASM compilation tracking
# Shows trunk installation and Leptos build progress with detailed colorization
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

# Progress tracking
COMPILE_COUNT=0
TOTAL_CRATES=0
TRUNK_CRATES=0
LEPTOS_CRATES=0
START_TIME=$(date +%s)
CURRENT_PHASE="trunk-install"

echo -e "${BOLD}${PURPLE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${PURPLE}║   Leptos Frontend Build Monitor - Real-Time WASM Build    ║${NC}"
echo -e "${BOLD}${PURPLE}║   Press Ctrl+C to exit                                     ║${NC}"
echo -e "${BOLD}${PURPLE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${MAGENTA}Monitoring: ${NC}docker compose build frontend-leptos"
echo -e "${MAGENTA}Started: ${NC}$(date)"
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
    printf "${PURPLE}["
    printf "%${filled}s" | tr ' ' '█'
    printf "%${empty}s" | tr ' ' '░'
    printf "] ${percent}%% (${COMPILE_COUNT}/${TOTAL_CRATES})${NC} "
  fi
}

# Function to detect build phase
detect_phase() {
  local line="$1"

  if echo "$line" | grep -q "Installing trunk"; then
    CURRENT_PHASE="trunk-install"
    echo -e "${BOLD}${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║           Phase 1: Installing Trunk Build Tool           ║${NC}"
    echo -e "${BOLD}${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
  elif echo "$line" | grep -q "rustup target add wasm32"; then
    CURRENT_PHASE="wasm-target"
    echo ""
    echo -e "${BOLD}${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║         Phase 2: Adding WASM32 Target to Rustup          ║${NC}"
    echo -e "${BOLD}${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
  elif echo "$line" | grep -q "trunk build"; then
    CURRENT_PHASE="leptos-build"
    COMPILE_COUNT=0
    TOTAL_CRATES=0
    echo ""
    echo -e "${BOLD}${PURPLE}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${PURPLE}║        Phase 3: Building Leptos WASM Application         ║${NC}"
    echo -e "${BOLD}${PURPLE}╚══════════════════════════════════════════════════════════╝${NC}"
  elif echo "$line" | grep -q "COPY --from=builder"; then
    CURRENT_PHASE="nginx-stage"
    echo ""
    echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║      Phase 4: Creating Nginx Production Container        ║${NC}"
    echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
  fi
}

# Function to colorize compilation output
colorize_build() {
  local line="$1"

  # Detect phase changes
  detect_phase "$line"

  # Track total crates from Locking message
  if echo "$line" | grep -q "Locking.*packages"; then
    local count=$(echo "$line" | grep -oP '\d+(?= packages)')
    TOTAL_CRATES=$count
    if [ "$CURRENT_PHASE" = "trunk-install" ]; then
      TRUNK_CRATES=$count
    elif [ "$CURRENT_PHASE" = "leptos-build" ]; then
      LEPTOS_CRATES=$count
    fi
    echo -e "${MAGENTA}📦 Locking $count packages...${NC}"

  # Trunk installation progress
  elif echo "$line" | grep -q "Installing trunk"; then
    echo -e "${BOLD}${CYAN}🔧 Installing trunk v0.21.14...${NC}"

  # Compiling trunk itself
  elif echo "$line" | grep -q "Compiling trunk v"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    show_progress
    echo -e "${BOLD}${CYAN}>>> COMPILING TRUNK BUILD TOOL${NC} $(show_elapsed)"

  # Compiling Leptos frontend
  elif echo "$line" | grep -q "Compiling.*frontend-leptos"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    show_progress
    echo -e "${BOLD}${PURPLE}🦀 >>> COMPILING LEPTOS FRONTEND${NC} $(show_elapsed)"

  # Compiling Leptos dependencies
  elif echo "$line" | grep -qE "Compiling (leptos|leptos_meta|leptos_router|gloo-net|wasm-bindgen)"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    PKG=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\) \(v[^ ]*\).*/\1 \2/p')
    show_progress
    echo -e "${BOLD}${PURPLE}🌐 $PKG ${DIM}(WASM core)${NC} $(show_elapsed)"

  # Regular compilation
  elif echo "$line" | grep -q "Compiling"; then
    COMPILE_COUNT=$((COMPILE_COUNT + 1))
    PKG=$(echo "$line" | sed -n 's/.*Compiling \([^ ]*\) \(v[^ ]*\).*/\1 \2/p')
    if [ -n "$PKG" ]; then
      show_progress
      echo -e "${GREEN}⚙  $PKG${NC}"
    else
      echo -e "${GREEN}$line${NC}"
    fi

  # Trunk build stages
  elif echo "$line" | grep -q "Finished.*trunk"; then
    echo -e "${BOLD}${GREEN}✓ Trunk compilation completed!${NC}"

  elif echo "$line" | grep -q "Building.*trunk"; then
    echo -e "${CYAN}🔨 Building with trunk...${NC}"

  elif echo "$line" | grep -q "info: downloading component 'rust-std' for 'wasm32-unknown-unknown'"; then
    echo -e "${CYAN}⬇  Downloading WASM32 standard library...${NC}"

  elif echo "$line" | grep -q "info: installing component 'rust-std' for 'wasm32-unknown-unknown'"; then
    echo -e "${GREEN}📦 Installing WASM32 standard library...${NC}"

  # WASM optimization
  elif echo "$line" | grep -qi "wasm-opt\|optimizing.*wasm"; then
    echo -e "${PURPLE}⚡ Optimizing WASM bundle for production...${NC}"

  elif echo "$line" | grep -qi "wasm-bindgen"; then
    echo -e "${PURPLE}🔗 Generating JavaScript bindings...${NC}"

  # Build success
  elif echo "$line" | grep -q "Finished.*release"; then
    local elapsed=$(($(date +%s) - START_TIME))
    echo ""
    echo -e "${BOLD}${PURPLE}══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${GREEN}✓✓✓ BUILD SUCCESSFUL ✓✓✓${NC}"
    echo -e "${CYAN}Time: ${elapsed}s | Crates compiled: ${COMPILE_COUNT}${NC}"
    if [ $TRUNK_CRATES -gt 0 ]; then
      echo -e "${CYAN}Trunk dependencies: ${TRUNK_CRATES} | Leptos app: ${LEPTOS_CRATES}${NC}"
    fi
    echo -e "${BOLD}${PURPLE}══════════════════════════════════════════════════════${NC}"

  elif echo "$line" | grep -q "Finished"; then
    echo -e "${CYAN}✓ $line${NC}"

  # Docker build stages
  elif echo "$line" | grep -q "^\[builder"; then
    echo -e "${BLUE}$line${NC}"

  elif echo "$line" | grep -q "^\[stage-"; then
    echo -e "${BLUE}$line${NC}"

  # Errors
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

  # Warnings
  elif echo "$line" | grep -qi "warning\["; then
    echo -e "${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "^warning:"; then
    echo -e "${YELLOW}⚠  $line${NC}"

  elif echo "$line" | grep -qi "warning"; then
    echo -e "${YELLOW}$line${NC}"

  # Downloading/fetching
  elif echo "$line" | grep -q "Downloaded\|Downloading"; then
    echo -e "${DIM}${BLUE}⬇  $line${NC}"

  elif echo "$line" | grep -q "Fetching\|Updating crates.io"; then
    echo -e "${BLUE}🔄 $line${NC}"

  # Build success indicator
  elif echo "$line" | grep -q "Successfully built\|Successfully tagged"; then
    local elapsed=$(($(date +%s) - START_TIME))
    local mins=$((elapsed / 60))
    local secs=$((elapsed % 60))
    echo ""
    echo -e "${BOLD}${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║           🎉 LEPTOS FRONTEND BUILD COMPLETE 🎉           ║${NC}"
    echo -e "${BOLD}${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${GREEN}$line${NC}"
    echo -e "${CYAN}Total build time: ${mins}m ${secs}s${NC}"
    echo -e "${CYAN}Access your app at: ${BOLD}http://localhost:8081${NC}"
    echo ""

  # Failed to solve
  elif echo "$line" | grep -qi "failed to solve"; then
    echo ""
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                      BUILD FAILED                         ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo -e "${BOLD}${RED}$line${NC}"

  # Dimmed patterns
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
  echo -e "  ${CYAN}# In one terminal, start the build:${NC}"
  echo -e "  docker compose build frontend-leptos"
  echo ""
  echo -e "  ${CYAN}# In another terminal, monitor it:${NC}"
  echo -e "  docker compose logs -f frontend-leptos 2>&1 | ./monitor-leptos-build.sh"
  echo ""
  echo -e "  ${CYAN}# Or pipe directly:${NC}"
  echo -e "  docker compose build frontend-leptos 2>&1 | ./monitor-leptos-build.sh"
  exit 1
fi

# Show summary on exit
trap 'echo ""; echo -e "${BOLD}${PURPLE}Leptos build monitoring stopped${NC}"' EXIT
