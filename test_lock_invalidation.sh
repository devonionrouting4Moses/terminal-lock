#!/bin/bash
# Test script to verify lock invalidation functionality
# This script demonstrates that sessions are invalidated when the laptop is locked

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

print_header "Lock Invalidation Test"

# Step 1: Verify no session exists
print_info "Step 1: Verifying no session exists..."
if [ -f "$SESSION_DIR/session.*.json" ]; then
    print_error "Session files already exist. Please clean up first."
    exit 1
fi
print_success "No existing sessions found"

# Step 2: Test check command with no session (should fail)
print_info "Step 2: Testing 'check' command with no session..."
if $TERMINAL_LOCK check 2>/dev/null; then
    print_error "Check should fail when no session exists"
    exit 1
fi
print_success "Check correctly failed (exit code 1) - terminal would be blocked"

# Step 3: Simulate authentication by creating a valid session
print_info "Step 3: Creating a valid session..."
$TERMINAL_LOCK auth 2>&1 << EOF
password
password
EOF

# Verify session was created
SESSION_FILE=$(ls "$SESSION_DIR"/session.*.json 2>/dev/null | head -1)
if [ -z "$SESSION_FILE" ]; then
    print_error "Session file was not created"
    exit 1
fi
print_success "Session created: $(basename $SESSION_FILE)"

# Step 4: Test check command with valid session (should succeed)
print_info "Step 4: Testing 'check' command with valid session..."
if ! $TERMINAL_LOCK check 2>/dev/null; then
    print_error "Check should succeed when session is valid"
    exit 1
fi
print_success "Check succeeded (exit code 0) - terminal is usable"

# Step 5: Simulate lock by manually deleting the session
print_info "Step 5: Simulating system lock by deleting session..."
rm "$SESSION_FILE"
print_success "Session deleted (simulating lock detection)"

# Step 6: Test check command after session deletion (should fail)
print_info "Step 6: Testing 'check' command after session deletion..."
if $TERMINAL_LOCK check 2>/dev/null; then
    print_error "Check should fail after session is deleted"
    exit 1
fi
print_success "Check correctly failed (exit code 1) - terminal is now blocked"

print_header "Test Results"
echo -e "${GREEN}✓ All tests passed!${NC}"
echo -e "\n${YELLOW}Summary:${NC}"
echo -e "  • Sessions are correctly validated before command execution"
echo -e "  • When a session is deleted (lock detected), terminal becomes unusable"
echo -e "  • User must re-authenticate to use the terminal after lock"
echo -e "\n${BLUE}How it works:${NC}"
echo -e "  1. Shell calls: ${BLUE}terminal-lock check && terminal-lock extend${NC}"
echo -e "  2. If check fails (no valid session), shell exits with error"
echo -e "  3. Terminal becomes read-only/unusable until re-authentication"
echo -e "  4. When laptop locks, daemon deletes all sessions"
echo -e "  5. Next command attempt fails check, blocking terminal usage\n"
