#!/bin/bash
set -e

echo "🔨 Building Terminal Lock..."
#cd terminal-lock

# Build release binaries
cargo build --release 2>&1 | tail -20

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 Binaries:"
ls -lh target/release/terminal-lock target/release/terminal-lock-daemon 2>/dev/null || echo "Binaries not found"

echo ""
echo "📋 Installation steps:"
echo "1. cp target/release/terminal-lock ~/.local/bin/"
echo "2. cp target/release/terminal-lock-daemon ~/.local/bin/"
echo "3. chmod +x ~/.local/bin/terminal-lock*"
echo "4. cp terminal-lock-daemon.service ~/.config/systemd/user/"
echo "5. systemctl --user daemon-reload"
echo "6. systemctl --user enable terminal-lock-daemon"
echo "7. systemctl --user start terminal-lock-daemon"
echo ""
echo "✅ Then test:"
echo "   terminal-lock auth"
