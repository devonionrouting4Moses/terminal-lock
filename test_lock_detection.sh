#!/bin/bash

# Step-by-step test for lock detection
TERMINAL_LOCK="./target/release/terminal-lock"

echo "🧪 Terminal Lock Detection Test"
echo "==============================="
echo ""

echo "📋 Step 1: Clean Environment"
echo "-----------------------------"
echo "Cleaning all session files..."
rm -f ~/.terminal_lock/session.*.json ~/.terminal_lock/activity.*.txt
echo "✅ Session files cleaned"
echo "Current PID: $$"
echo ""

echo "🚀 Step 2: Start Lock Monitor Daemon"
echo "------------------------------------"
echo "Starting daemon in background..."
$TERMINAL_LOCK daemon &
DAEMON_PID=$!
echo "✅ Daemon started with PID: $DAEMON_PID"
sleep 2
echo ""

echo "🔐 Step 3: Authenticate"
echo "----------------------"
echo "Running authentication..."
if $TERMINAL_LOCK auth; then
    echo "✅ Authentication successful"
else
    echo "❌ Authentication failed"
    kill $DAEMON_PID 2>/dev/null
    exit 1
fi
echo ""

echo "📂 Step 4: Check Session File"
echo "-----------------------------"
SESSION_FILE="$HOME/.terminal_lock/session.$$.json"
if [ -f "$SESSION_FILE" ]; then
    echo "✅ Session file created: $SESSION_FILE"
    echo "Session contents:"
    cat "$SESSION_FILE"
else
    echo "❌ No session file found"
    kill $DAEMON_PID 2>/dev/null
    exit 1
fi
echo ""

echo "⚡ Step 5: Test Commands (Simulate Activity)"
echo "--------------------------------------------"
echo "Running test commands to simulate terminal activity..."
echo "Command 1: ls"
ls > /dev/null
$TERMINAL_LOCK extend
echo "Command 2: pwd"
pwd > /dev/null
$TERMINAL_LOCK extend
echo "Command 3: uname -r"
uname -r > /dev/null
$TERMINAL_LOCK extend
echo "Command 4: whoami"
whoami > /dev/null
$TERMINAL_LOCK extend
echo "Command 5: date"
date > /dev/null
$TERMINAL_LOCK extend
echo "✅ All commands executed with session extension"
echo ""

echo "📊 Step 6: Check Updated Session"
echo "--------------------------------"
if [ -f "$SESSION_FILE" ]; then
    echo "Session file after activity:"
    cat "$SESSION_FILE"
else
    echo "❌ Session file disappeared"
fi
echo ""

echo "🔒 Step 7: MANUAL TEST REQUIRED"
echo "-------------------------------"
echo "🚨 NOW LOCK YOUR LAPTOP/COMPUTER!"
echo ""
echo "Instructions:"
echo "1. Lock your screen (Super+L or Ctrl+Alt+L)"
echo "2. Wait 3-5 seconds"
echo "3. Unlock your screen"
echo "4. Come back to this terminal"
echo "5. Press Enter to continue the test"
echo ""
read -p "Press Enter after you have locked and unlocked your screen..."
echo ""

echo "🔍 Step 8: Check Session After Lock"
echo "-----------------------------------"
if [ -f "$SESSION_FILE" ]; then
    echo "❌ FAILED: Session file still exists after lock!"
    echo "Session contents:"
    cat "$SESSION_FILE"
    echo ""
    echo "This means lock detection is NOT working."
else
    echo "✅ SUCCESS: Session file was deleted after lock!"
    echo "Lock detection is working correctly."
fi
echo ""

echo "🧹 Step 9: Cleanup"
echo "------------------"
echo "Stopping daemon..."
kill $DAEMON_PID 2>/dev/null
echo "✅ Daemon stopped"
echo ""

echo "📋 Test Summary"
echo "==============="
if [ -f "$SESSION_FILE" ]; then
    echo "❌ LOCK DETECTION FAILED"
    echo "   - Session persisted after system lock"
    echo "   - Need to investigate daemon monitoring"
else
    echo "✅ LOCK DETECTION WORKING"
    echo "   - Session correctly invalidated on system lock"
    echo "   - Security feature functioning properly"
fi
echo ""
echo "Test completed."
