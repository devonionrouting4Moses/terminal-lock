#!/bin/bash
# Debug script to track lock monitor behavior
# Run this in a separate terminal to see what's happening

echo "🔍 Terminal Lock Debug Monitor"
echo "================================"
echo ""

CONFIG_DIR="$HOME/.terminal-lock"

# Monitor lock generation file
echo "📊 Monitoring lock generation counter..."
watch -n 0.5 "
echo '=== Lock Generation ==='
if [ -f '$CONFIG_DIR/lock_generation' ]; then
    echo 'Current generation: '
    cat '$CONFIG_DIR/lock_generation'
else
    echo 'No lock generation file (counter = 0)'
fi
echo ''

echo '=== Session Files ==='
ls -lh '$CONFIG_DIR'/session.*.json 2>/dev/null || echo 'No session files'
echo ''

echo '=== Session Contents ==='
for f in '$CONFIG_DIR'/session.*.json; do
    if [ -f \"\$f\" ]; then
        echo \"File: \$f\"
        cat \"\$f\" | jq '.' 2>/dev/null || cat \"\$f\"
        echo ''
    fi
done

echo '=== GNOME Session Status ==='
dbus-send --session --print-reply \
    --dest=org.gnome.SessionManager \
    /org/gnome/SessionManager/Presence \
    org.freedesktop.DBus.Properties.Get \
    string:org.gnome.SessionManager.Presence \
    string:status 2>/dev/null | grep uint32 || echo 'D-Bus query failed'
echo ''
echo 'Status codes:'
echo '  0 = available (unlocked)'
echo '  1 = invisible (LOCKED)'
echo '  2 = busy'
echo '  3 = idle'
"