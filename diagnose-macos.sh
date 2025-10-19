#!/bin/bash
# MultiShiva macOS Diagnostic Script

echo "🍎 MultiShiva macOS Diagnostic"
echo "================================"
echo

# 1. Check macOS version
echo "1. macOS Version:"
sw_vers
echo

# 2. Check network connectivity to host
echo "2. Network Test:"
read -p "Enter host IP address (e.g., 192.168.1.100): " HOST_IP
read -p "Enter port (default: 53421): " PORT
PORT=${PORT:-53421}

echo "Testing connection to $HOST_IP:$PORT..."
if nc -zv $HOST_IP $PORT 2>&1 | grep -q succeeded; then
    echo "✅ Network connection successful"
else
    echo "❌ Network connection failed"
    echo "   Possible causes:"
    echo "   - Host is not running"
    echo "   - Firewall blocking connection"
    echo "   - Wrong IP address or port"
fi
echo

# 3. Check firewall status
echo "3. Firewall Status:"
FW_STATE=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)
echo "$FW_STATE"
echo

# 4. Check Accessibility permissions for Terminal
echo "4. Accessibility Permissions:"
echo "Checking if Terminal/iTerm has Accessibility access..."
echo "You need to manually verify in:"
echo "  System Settings → Privacy & Security → Accessibility"
echo
echo "Current shell: $SHELL"
echo "Terminal app: $(ps -o comm= -p $PPID)"
echo

# 5. Check if cargo/rust is available
echo "5. Rust Environment:"
if command -v cargo &> /dev/null; then
    echo "✅ Cargo found: $(cargo --version)"
else
    echo "❌ Cargo not found"
fi
echo

# 6. Test MultiShiva permissions check
echo "6. MultiShiva Permission Check:"
if [ -f "target/debug/multishiva" ] || [ -f "target/release/multishiva" ]; then
    BINARY=$(find target -name multishiva -type f | head -1)
    echo "Found binary: $BINARY"
    echo "Running permission check..."
    # This would run the permission check
else
    echo "ℹ️  No compiled binary found. Run 'cargo build' first."
fi
echo

echo "📋 Next Steps:"
echo "1. Ensure Terminal has Accessibility permissions"
echo "2. Check firewall allows outgoing connections"
echo "3. Verify host is running: cargo run -- -m host"
echo "4. Try connecting: cargo run -- -m agent --host $HOST_IP:$PORT"
echo
