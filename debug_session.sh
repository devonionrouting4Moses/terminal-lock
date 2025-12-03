#!/bin/bash

# Debug session validation
export HOME="/tmp/terminal-lock-test"
export TERMINAL_LOCK_DIR="/tmp/terminal-lock-test/.terminal_lock"
TERMINAL_LOCK="./target/release/terminal-lock"

# Clean up
rm -rf "$HOME"
mkdir -p "$TERMINAL_LOCK_DIR"

# Get current PID
PID=$$
SESSION_FILE="$TERMINAL_LOCK_DIR/session.$PID.json"

echo "Debug Session Validation"
echo "========================"
echo "PID: $PID"
echo "Session file: $SESSION_FILE"
echo ""

# Create expired session (16 minutes ago)
EXPIRED_TIME=$(date -u -d '16 minutes ago' --iso-8601=seconds)
RECENT_ACTIVITY=$(date -u -d '1 minute ago' --iso-8601=seconds)

cat > "$SESSION_FILE" << EOF
{
  "expiry": "$EXPIRED_TIME",
  "last_activity": "$RECENT_ACTIVITY",
  "terminal_id": "$PID"
}
EOF

echo "Created session file:"
cat "$SESSION_FILE"
echo ""

echo "Current time: $(date -u --iso-8601=seconds)"
echo "Session expiry: $EXPIRED_TIME"
echo "Time difference: $(( $(date -d "now" +%s) - $(date -d "$EXPIRED_TIME" +%s) )) seconds"
echo ""

echo "Running extend command..."
$TERMINAL_LOCK extend

echo ""
echo "After extend command:"
if [ -f "$SESSION_FILE" ]; then
    echo "❌ Session file still exists:"
    cat "$SESSION_FILE"
else
    echo "✅ Session file was deleted"
fi
