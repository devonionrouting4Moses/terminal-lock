#!/bin/bash

echo "🔍 D-Bus Diagnostic Test"
echo "========================"
echo ""

# Test 1: Check if D-Bus is running
echo "1. D-Bus Service Status:"
echo "------------------------"
if systemctl --user is-active dbus >/dev/null 2>&1; then
    echo "✅ User D-Bus service is active"
else
    echo "❌ User D-Bus service is not active"
    echo "   Trying to start it..."
    systemctl --user start dbus 2>/dev/null || echo "   Failed to start user D-Bus"
fi

if systemctl is-active dbus >/dev/null 2>&1; then
    echo "✅ System D-Bus service is active"
else
    echo "❌ System D-Bus service is not active"
fi

echo ""

# Test 2: Check D-Bus environment variables
echo "2. D-Bus Environment:"
echo "---------------------"
echo "DBUS_SESSION_BUS_ADDRESS: ${DBUS_SESSION_BUS_ADDRESS:-'Not set'}"
echo "XDG_RUNTIME_DIR: ${XDG_RUNTIME_DIR:-'Not set'}"
echo "DISPLAY: ${DISPLAY:-'Not set'}"
echo ""

# Test 3: Check if dbus-send is available
echo "3. D-Bus Tools:"
echo "---------------"
if command -v dbus-send >/dev/null 2>&1; then
    echo "✅ dbus-send is available: $(which dbus-send)"
else
    echo "❌ dbus-send is not available"
    echo "   Install with: sudo apt install dbus-x11"
fi

if command -v dbus-monitor >/dev/null 2>&1; then
    echo "✅ dbus-monitor is available: $(which dbus-monitor)"
else
    echo "❌ dbus-monitor is not available"
fi

echo ""

# Test 4: Test D-Bus connection
echo "4. D-Bus Connection Test:"
echo "-------------------------"
if dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames >/dev/null 2>&1; then
    echo "✅ D-Bus session connection works"
else
    echo "❌ D-Bus session connection failed"
    echo "   Error details:"
    dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.ListNames 2>&1 | head -5
fi

echo ""

# Test 5: Check for ScreenSaver interface
echo "5. ScreenSaver Interface Test:"
echo "------------------------------"
if dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver /ScreenSaver org.freedesktop.ScreenSaver.GetActive >/dev/null 2>&1; then
    echo "✅ ScreenSaver interface is available"
    SCREENSAVER_ACTIVE=$(dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver /ScreenSaver org.freedesktop.ScreenSaver.GetActive 2>/dev/null | grep boolean | awk '{print $2}')
    echo "   Current state: $SCREENSAVER_ACTIVE"
else
    echo "❌ ScreenSaver interface is not available"
    echo "   Error details:"
    dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver /ScreenSaver org.freedesktop.ScreenSaver.GetActive 2>&1 | head -3
fi

echo ""

# Test 6: Check desktop environment
echo "6. Desktop Environment:"
echo "-----------------------"
echo "XDG_CURRENT_DESKTOP: ${XDG_CURRENT_DESKTOP:-'Not set'}"
echo "DESKTOP_SESSION: ${DESKTOP_SESSION:-'Not set'}"
echo "GDMSESSION: ${GDMSESSION:-'Not set'}"

# Check for specific desktop screensavers
if command -v gnome-screensaver-command >/dev/null 2>&1; then
    echo "✅ GNOME screensaver available"
    gnome-screensaver-command --query 2>/dev/null || echo "   (query failed)"
elif command -v xscreensaver-command >/dev/null 2>&1; then
    echo "✅ XScreenSaver available"
elif command -v kscreenlocker_greet >/dev/null 2>&1; then
    echo "✅ KDE screen locker available"
else
    echo "❌ No known screensaver found"
fi

echo ""

# Test 7: Alternative D-Bus services
echo "7. Alternative Lock Detection:"
echo "------------------------------"

# Check for login1 interface (systemd)
if dbus-send --system --print-reply --dest=org.freedesktop.login1 /org/freedesktop/login1 org.freedesktop.login1.Manager.ListSessions >/dev/null 2>&1; then
    echo "✅ systemd login1 interface available"
else
    echo "❌ systemd login1 interface not available"
fi

# Check for UPower (can detect suspend/resume)
if dbus-send --system --print-reply --dest=org.freedesktop.UPower /org/freedesktop/UPower org.freedesktop.UPower.EnumerateDevices >/dev/null 2>&1; then
    echo "✅ UPower interface available"
else
    echo "❌ UPower interface not available"
fi

echo ""
echo "🔧 Recommendations:"
echo "==================="

if [ -z "$DBUS_SESSION_BUS_ADDRESS" ]; then
    echo "• Set up D-Bus session: eval \$(dbus-launch --sh-syntax)"
fi

if ! command -v dbus-send >/dev/null 2>&1; then
    echo "• Install D-Bus tools: sudo apt install dbus-x11"
fi

if [ -z "$DISPLAY" ]; then
    echo "• Running in non-graphical environment - D-Bus ScreenSaver won't work"
    echo "• Polling mode is the correct fallback"
fi

echo ""
echo "🧪 Quick D-Bus Monitor Test:"
echo "=============================="
echo "Starting 5-second D-Bus monitor test..."
echo "Lock/unlock your screen now to test detection!"
echo ""

timeout 5 dbus-monitor "type='signal',interface='org.freedesktop.ScreenSaver',member='ActiveChanged'" 2>/dev/null || echo "D-Bus monitor test failed or no signals received"

echo ""
echo "Test completed."
