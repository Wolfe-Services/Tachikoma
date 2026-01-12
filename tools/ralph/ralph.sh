#!/usr/bin/env bash
#
# Ralph Wiggum Loop - Shell Script Implementation
#
# "It's not that hard to build a coding agent. It's 300 lines of code
# running in a loop with LLM tokens. The model does all the heavy lifting."
# - Geoffrey Huntley
#
# This is a minimal shell implementation that uses Claude CLI.
# For the full Rust implementation, install cargo and run: cargo build --release
#

set -euo pipefail

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
SPECS_DIR="${PROJECT_ROOT}/specs"
PROMPT_FILE="${PROJECT_ROOT}/prompt.md"
MAX_ITERATIONS="${MAX_ITERATIONS:-100}"
STOP_ON_FAIL_STREAK="${STOP_ON_FAIL_STREAK:-3}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║                                                           ║"
    echo "║   ████████╗ █████╗  ██████╗██╗  ██╗██╗██╗  ██╗ ██████╗   ║"
    echo "║   ╚══██╔══╝██╔══██╗██╔════╝██║  ██║██║██║ ██╔╝██╔═══██╗  ║"
    echo "║      ██║   ███████║██║     ███████║██║█████╔╝ ██║   ██║  ║"
    echo "║      ██║   ██╔══██║██║     ██╔══██║██║██╔═██╗ ██║   ██║  ║"
    echo "║      ██║   ██║  ██║╚██████╗██║  ██║██║██║  ██╗╚██████╔╝  ║"
    echo "║      ╚═╝   ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═╝ ╚═════╝  ║"
    echo "║                                                           ║"
    echo "║              Ralph Wiggum Loop - Agentic Harness          ║"
    echo "║                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prereqs() {
    if ! command -v claude &> /dev/null; then
        log_error "claude CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
        exit 1
    fi

    if [[ ! -d "$SPECS_DIR" ]]; then
        log_error "specs/ directory not found at $SPECS_DIR"
        exit 1
    fi

    if [[ ! -f "$SPECS_DIR/README.md" ]]; then
        log_error "specs/README.md (THE PIN) not found"
        exit 1
    fi
}

# Find next uncompleted spec from README.md
find_next_spec() {
    # Parse the README.md to find spec entries
    # Look for lines like: | 001 | [Project Structure](phase-00-setup/001-project-structure.md) |

    local readme="$SPECS_DIR/README.md"

    # Extract spec lines and process them in order
    grep -E '^\|\s*[0-9]{3}\s*\|' "$readme" | while IFS='|' read -r _ id name_link _; do
        # Clean up the ID
        id=$(echo "$id" | tr -d ' ')

        # Extract path from markdown link
        local path=$(echo "$name_link" | sed -n 's/.*](\([^)]*\)).*/\1/p')

        if [[ -z "$path" ]]; then
            continue
        fi

        local full_path="$SPECS_DIR/$path"

        if [[ ! -f "$full_path" ]]; then
            continue
        fi

        # Check if all acceptance criteria are complete
        # Count unchecked boxes in acceptance criteria section
        local unchecked=$(sed -n '/## Acceptance Criteria/,/^## /p' "$full_path" | grep -c '\- \[ \]' || true)

        if [[ "$unchecked" -gt 0 ]]; then
            echo "$id:$full_path"
            return 0
        fi
    done

    echo ""
}

# Get progress summary
get_progress() {
    local total=0
    local completed=0

    grep -E '^\|\s*[0-9]{3}\s*\|' "$SPECS_DIR/README.md" | while IFS='|' read -r _ id name_link _; do
        local path=$(echo "$name_link" | sed -n 's/.*](\([^)]*\)).*/\1/p')

        if [[ -z "$path" ]]; then
            continue
        fi

        local full_path="$SPECS_DIR/$path"

        if [[ ! -f "$full_path" ]]; then
            continue
        fi

        ((total++)) || true

        local unchecked=$(sed -n '/## Acceptance Criteria/,/^## /p' "$full_path" | grep -c '\- \[ \]' || true)

        if [[ "$unchecked" -eq 0 ]]; then
            ((completed++)) || true
        fi
    done

    echo "$completed/$total"
}

# Generate prompt for a spec
generate_prompt() {
    local spec_path="$1"
    local spec_id="$2"

    cat << EOF
Study specs/README.md to understand the project structure and spec index.

Your current mission: Implement spec $spec_id

Spec file: $spec_path

Instructions:
1. Read the spec file completely
2. Follow all acceptance criteria
3. Implement the code according to the patterns specified
4. Run tests if applicable
5. Update each acceptance criterion checkbox from "- [ ]" to "- [x]" when complete
6. When all criteria are complete, summarize what was done

Constraints:
- Follow existing patterns in the codebase
- Make small, focused changes
- Test your changes before marking complete

Begin by reading the spec file.
EOF
}

# Run a single spec implementation
run_spec() {
    local spec_info="$1"
    local spec_id="${spec_info%%:*}"
    local spec_path="${spec_info#*:}"

    log_info "Starting spec $spec_id: $spec_path"

    # Generate prompt
    local prompt=$(generate_prompt "$spec_path" "$spec_id")

    # Write to prompt.md
    echo "$prompt" > "$PROMPT_FILE"

    log_info "Prompt written to $PROMPT_FILE"
    log_info "Running Claude..."

    # Run Claude with the prompt
    # Using --dangerously-skip-permissions for unattended operation
    if cat "$PROMPT_FILE" | claude --dangerously-skip-permissions; then
        log_success "Claude completed for spec $spec_id"

        # Auto-commit changes
        if git -C "$PROJECT_ROOT" status --short | grep -q .; then
            log_info "Committing changes..."
            git -C "$PROJECT_ROOT" add -A
            git -C "$PROJECT_ROOT" commit -m "spec($spec_id): implement spec

Automated commit by Ralph loop."
            log_success "Changes committed"
        else
            log_info "No changes to commit"
        fi

        return 0
    else
        log_error "Claude failed for spec $spec_id"
        return 1
    fi
}

# Main loop
run_loop() {
    local fail_streak=0
    local specs_completed=0
    local iteration=0

    while [[ $iteration -lt $MAX_ITERATIONS ]]; do
        ((iteration++)) || true

        log_info "=== Iteration $iteration / $MAX_ITERATIONS ==="

        # Find next spec
        local next=$(find_next_spec)

        if [[ -z "$next" ]]; then
            log_success "All specs are complete!"
            break
        fi

        local spec_id="${next%%:*}"
        log_info "Next spec: $spec_id"

        # Run the spec
        if run_spec "$next"; then
            ((specs_completed++)) || true
            fail_streak=0
        else
            ((fail_streak++)) || true

            if [[ $fail_streak -ge $STOP_ON_FAIL_STREAK ]]; then
                log_error "Stopping: $fail_streak consecutive failures"
                break
            fi
        fi

        # Brief pause between iterations
        sleep 2
    done

    echo ""
    log_info "========================================="
    log_info "  Loop Complete"
    log_info "  Iterations: $iteration"
    log_info "  Specs completed: $specs_completed"
    log_info "========================================="
}

# Show status
show_status() {
    print_banner

    log_info "Project: $PROJECT_ROOT"
    log_info "Specs directory: $SPECS_DIR"
    echo ""

    # Count specs
    local total=$(grep -cE '^\|\s*[0-9]{3}\s*\|' "$SPECS_DIR/README.md" || echo "0")
    log_info "Total specs: $total"

    # Find next
    local next=$(find_next_spec)
    if [[ -n "$next" ]]; then
        local next_id="${next%%:*}"
        log_info "Next spec: $next_id"
    else
        log_success "All specs complete!"
    fi
}

# Show next spec
show_next() {
    local next=$(find_next_spec)

    if [[ -z "$next" ]]; then
        log_success "All specs are complete!"
        return 0
    fi

    local spec_id="${next%%:*}"
    local spec_path="${next#*:}"

    echo ""
    echo "Next Spec: $spec_id"
    echo "Path: $spec_path"
    echo ""
    echo "Acceptance Criteria:"
    sed -n '/## Acceptance Criteria/,/^## /p' "$spec_path" | grep -E '^\s*-\s*\[' || true
    echo ""
}

# Usage
usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  run       Run single iteration (implement one spec)"
    echo "  loop      Run continuous loop until all specs complete"
    echo "  status    Show current progress"
    echo "  next      Show next spec to implement"
    echo ""
    echo "Environment variables:"
    echo "  PROJECT_ROOT           Project root directory (default: current dir)"
    echo "  MAX_ITERATIONS         Maximum loop iterations (default: 100)"
    echo "  STOP_ON_FAIL_STREAK    Stop after N consecutive failures (default: 3)"
    echo ""
}

# Main
main() {
    local cmd="${1:-status}"

    check_prereqs

    case "$cmd" in
        run)
            print_banner
            local next=$(find_next_spec)
            if [[ -z "$next" ]]; then
                log_success "All specs are complete!"
                exit 0
            fi
            run_spec "$next"
            ;;
        loop)
            print_banner
            run_loop
            ;;
        status)
            show_status
            ;;
        next)
            show_next
            ;;
        help|-h|--help)
            usage
            ;;
        *)
            log_error "Unknown command: $cmd"
            usage
            exit 1
            ;;
    esac
}

main "$@"
