#!/bin/bash

# Test session validation within the same process
TERMINAL_LOCK="./target/release/terminal-lock"
LOCK_DIR="$HOME/.terminal_lock"

echo "🧪 Session Validation Test (Same Process)"
echo "========================================="
echo "Lock directory: $LOCK_DIR"
echo "Current PID: $$"
echo ""

# Backup existing session if any
SESSION_FILE="$LOCK_DIR/session.$$.json"
BACKUP_FILE="$LOCK_DIR/session.$$.json.backup"

if [ -f "$SESSION_FILE" ]; then
    echo "Backing up existing session..."
    cp "$SESSION_FILE" "$BACKUP_FILE"
fi

# Function to create session file
create_session() {
    local expiry="$1"
    local last_activity="$2"
    local description="$3"
    
    cat > "$SESSION_FILE" << EOF
{
  "expiry": "$expiry",
  "last_activity": "$last_activity",
  "terminal_id": "$$"
}
EOF
    
    echo "Created $description session:"
    echo "  - Expiry: $expiry"
    echo "  - Last activity: $last_activity"
    echo "  - PID: $$"
}

# Test 1: Expired session (16 minutes ago)
echo "🕐 Test 1: Expired Session (16 minutes ago)"
echo "-------------------------------------------"

EXPIRED_TIME=$(date -u -d '16 minutes ago' --iso-8601=seconds)
RECENT_ACTIVITY=$(date -u -d '1 minute ago' --iso-8601=seconds)

create_session "$EXPIRED_TIME" "$RECENT_ACTIVITY" "expired"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

echo "Checking result..."
if [ -f "$SESSION_FILE" ]; then
    echo "❌ FAILED: Session file should be deleted (15-minute timeout)"
    echo "Session file contents:"
    cat "$SESSION_FILE"
else
    echo "✅ SUCCESS: Expired session was correctly deleted"
fi
echo ""

# Test 2: Inactive session (6 minutes ago)
echo "💤 Test 2: Inactive Session (6 minutes ago)"
echo "-------------------------------------------"

VALID_EXPIRY=$(date -u -d '10 minutes' --iso-8601=seconds)
INACTIVE_TIME=$(date -u -d '6 minutes ago' --iso-8601=seconds)

create_session "$VALID_EXPIRY" "$INACTIVE_TIME" "inactive"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

echo "Checking result..."
if [ -f "$SESSION_FILE" ]; then
    echo "❌ FAILED: Session file should be deleted (5-minute inactivity timeout)"
    echo "Session file contents:"
    cat "$SESSION_FILE"
else
    echo "✅ SUCCESS: Inactive session was correctly deleted"
fi
echo ""

# Test 3: Valid active session
echo "✅ Test 3: Valid Active Session"
echo "------------------------------"

VALID_EXPIRY=$(date -u -d '10 minutes' --iso-8601=seconds)
RECENT_ACTIVITY=$(date -u -d '2 minutes ago' --iso-8601=seconds)

create_session "$VALID_EXPIRY" "$RECENT_ACTIVITY" "valid"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

echo "Checking result..."
if [ -f "$SESSION_FILE" ]; then
    echo "✅ SUCCESS: Valid session was maintained"
    echo "Updated session contents:"
    cat "$SESSION_FILE"
    echo ""
    
    # Check if activity was updated
    NEW_ACTIVITY=$(jq -r '.last_activity' "$SESSION_FILE" 2>/dev/null || echo "jq not available")
    NEW_EXPIRY=$(jq -r '.expiry' "$SESSION_FILE" 2>/dev/null || echo "jq not available")
    
    if [ "$NEW_EXPIRY" = "$VALID_EXPIRY" ]; then
        echo "✅ SUCCESS: Expiry time was preserved (no extension)"
    else
        echo "❌ FAILED: Expiry time was incorrectly modified"
        echo "   Expected: $VALID_EXPIRY"
        echo "   Got: $NEW_EXPIRY"
    fi
    
    # Check if activity timestamp was updated (should be more recent than 2 minutes ago)
    if [ "$NEW_ACTIVITY" != "$RECENT_ACTIVITY" ]; then
        echo "✅ SUCCESS: Activity timestamp was updated to: $NEW_ACTIVITY"
    else
        echo "❌ FAILED: Activity timestamp was not updated"
    fi
else
    echo "❌ FAILED: Valid session was unexpectedly deleted"
fi

# Cleanup - restore backup if it existed
echo ""
echo "🧹 Cleanup"
echo "----------"
if [ -f "$BACKUP_FILE" ]; then
    echo "Restoring original session..."
    mv "$BACKUP_FILE" "$SESSION_FILE"
else
    # Remove test session
    rm -f "$SESSION_FILE"
    echo "Test session removed"
fi

echo ""
echo "Test completed."
