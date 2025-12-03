#!/bin/bash

# Test session validation with real home directory
TERMINAL_LOCK="./target/release/terminal-lock"
LOCK_DIR="$HOME/.terminal_lock"

echo "🧪 Real Session Validation Test"
echo "==============================="
echo "Lock directory: $LOCK_DIR"
echo ""

# Backup existing session if any
PID=$$
SESSION_FILE="$LOCK_DIR/session.$PID.json"
BACKUP_FILE="$LOCK_DIR/session.$PID.json.backup"

if [ -f "$SESSION_FILE" ]; then
    echo "Backing up existing session..."
    cp "$SESSION_FILE" "$BACKUP_FILE"
fi

# Test 1: Expired session (16 minutes ago)
echo "🕐 Test 1: Expired Session (16 minutes ago)"
echo "-------------------------------------------"

EXPIRED_TIME=$(date -u -d '16 minutes ago' --iso-8601=seconds)
RECENT_ACTIVITY=$(date -u -d '1 minute ago' --iso-8601=seconds)

cat > "$SESSION_FILE" << EOF
{
  "expiry": "$EXPIRED_TIME",
  "last_activity": "$RECENT_ACTIVITY",
  "terminal_id": "$PID"
}
EOF

echo "Created expired session:"
echo "  - Expiry: $EXPIRED_TIME"
echo "  - Last activity: $RECENT_ACTIVITY"
echo "  - PID: $PID"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

if [ -f "$SESSION_FILE" ]; then
    echo "❌ FAILED: Session file should be deleted (15-minute timeout)"
    echo "Session file contents:"
    cat "$SESSION_FILE"
    echo ""
else
    echo "✅ SUCCESS: Expired session was correctly deleted"
    echo ""
fi

# Test 2: Inactive session (6 minutes ago)
echo "💤 Test 2: Inactive Session (6 minutes ago)"
echo "-------------------------------------------"

VALID_EXPIRY=$(date -u -d '10 minutes' --iso-8601=seconds)
INACTIVE_TIME=$(date -u -d '6 minutes ago' --iso-8601=seconds)

cat > "$SESSION_FILE" << EOF
{
  "expiry": "$VALID_EXPIRY",
  "last_activity": "$INACTIVE_TIME",
  "terminal_id": "$PID"
}
EOF

echo "Created inactive session:"
echo "  - Expiry: $VALID_EXPIRY (valid)"
echo "  - Last activity: $INACTIVE_TIME (6 minutes ago)"
echo "  - PID: $PID"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

if [ -f "$SESSION_FILE" ]; then
    echo "❌ FAILED: Session file should be deleted (5-minute inactivity timeout)"
    echo "Session file contents:"
    cat "$SESSION_FILE"
    echo ""
else
    echo "✅ SUCCESS: Inactive session was correctly deleted"
    echo ""
fi

# Test 3: Valid active session
echo "✅ Test 3: Valid Active Session"
echo "------------------------------"

VALID_EXPIRY=$(date -u -d '10 minutes' --iso-8601=seconds)
RECENT_ACTIVITY=$(date -u -d '2 minutes ago' --iso-8601=seconds)

cat > "$SESSION_FILE" << EOF
{
  "expiry": "$VALID_EXPIRY",
  "last_activity": "$RECENT_ACTIVITY",
  "terminal_id": "$PID"
}
EOF

echo "Created valid session:"
echo "  - Expiry: $VALID_EXPIRY (valid)"
echo "  - Last activity: $RECENT_ACTIVITY (2 minutes ago)"
echo "  - PID: $PID"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

if [ -f "$SESSION_FILE" ]; then
    echo "✅ SUCCESS: Valid session was maintained"
    echo "Updated session contents:"
    cat "$SESSION_FILE"
    echo ""
    
    # Check if activity was updated
    NEW_ACTIVITY=$(jq -r '.last_activity' "$SESSION_FILE")
    NEW_EXPIRY=$(jq -r '.expiry' "$SESSION_FILE")
    
    if [ "$NEW_EXPIRY" = "$VALID_EXPIRY" ]; then
        echo "✅ SUCCESS: Expiry time was preserved (no extension)"
    else
        echo "❌ FAILED: Expiry time was incorrectly modified"
    fi
    
    echo "Activity timestamp updated to: $NEW_ACTIVITY"
else
    echo "❌ FAILED: Valid session was unexpectedly deleted"
fi

# Cleanup - restore backup if it existed
if [ -f "$BACKUP_FILE" ]; then
    echo ""
    echo "Restoring original session..."
    mv "$BACKUP_FILE" "$SESSION_FILE"
else
    # Remove test session
    rm -f "$SESSION_FILE"
fi

echo ""
echo "Test completed."
