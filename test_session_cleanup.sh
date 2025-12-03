#!/bin/bash
# Test script to verify session cleanup on re-authentication
# This script verifies that old session files are deleted when creating a new session

set -e

TERMINAL_LOCK="$HOME/.local/bin/terminal-lock"
SESSION_DIR="$HOME/.terminal_lock"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Check if terminal-lock is installed
if [ ! -f "$TERMINAL_LOCK" ]; then
    print_error "terminal-lock not found at $TERMINAL_LOCK"
    exit 1
fi

print_header "Session Cleanup Test"

# Step 1: Clean up any existing sessions
print_info "Step 1: Cleaning up existing sessions..."
rm -f "$SESSION_DIR"/session.*.json
print_success "Cleaned up old sessions"

# Step 2: Create first session (simulate initial authentication)
print_info "Step 2: Creating first session (simulating initial authentication)..."
$TERMINAL_LOCK auth 2>&1 << EOF
password
password
EOF

SESSION_FILES=$(ls "$SESSION_DIR"/session.*.json 2>/dev/null | wc -l)
if [ "$SESSION_FILES" -ne 1 ]; then
    print_error "Expected 1 session file, found $SESSION_FILES"
    exit 1
fi
FIRST_SESSION=$(ls "$SESSION_DIR"/session.*.json | head -1)
print_success "First session created: $(basename $FIRST_SESSION)"

# Step 3: Simulate source ~/.zshrc by creating a subshell and authenticating again
print_info "Step 3: Simulating 'source ~/.zshrc' (re-authentication in subshell)..."
(
    $TERMINAL_LOCK auth 2>&1 << EOF
password
password
EOF
)

# Step 4: Check that only ONE session file exists
SESSION_FILES=$(ls "$SESSION_DIR"/session.*.json 2>/dev/null | wc -l)
if [ "$SESSION_FILES" -ne 1 ]; then
    print_error "Expected 1 session file after re-auth, found $SESSION_FILES"
    echo "Session files:"
    ls -la "$SESSION_DIR"/session.*.json 2>/dev/null || true
    exit 1
fi

SECOND_SESSION=$(ls "$SESSION_DIR"/session.*.json | head -1)
print_success "Only 1 session file exists after re-authentication"

# Step 5: Verify the old session was deleted
if [ "$FIRST_SESSION" = "$SECOND_SESSION" ]; then
    print_info "Session files have the same path (expected if PIDs are different)"
else
    print_success "Old session was replaced with new session"
fi

print_header "Test Results"
echo -e "${GREEN}✓ All tests passed!${NC}"
echo -e "\n${YELLOW}Summary:${NC}"
echo -e "  • Old session files are cleaned up when creating new sessions"
echo -e "  • Only one session file exists at a time per authentication"
echo -e "  • Re-sourcing shell config no longer creates duplicate sessions"
echo -e "\n${BLUE}How it works:${NC}"
echo -e "  1. User authenticates → Session created"
echo -e "  2. User runs 'source ~/.zshrc' → Re-authentication triggered"
echo -e "  3. Session::create() deletes old session files first"
echo -e "  4. New session created with current PID"
echo -e "  5. Only one session file exists\n"
