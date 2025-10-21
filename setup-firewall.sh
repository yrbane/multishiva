#!/bin/bash
# MultiShiva Firewall Setup Script
# Configures firewall to allow MultiShiva TCP and mDNS multicast

set -e

echo "🔥 MultiShiva Firewall Configuration"
echo "===================================="
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run as root (sudo ./setup-firewall.sh)"
    exit 1
fi

# Detect firewall system
if command -v firewall-cmd &> /dev/null; then
    FIREWALL="firewalld"
elif command -v ufw &> /dev/null; then
    FIREWALL="ufw"
else
    echo "⚠️  No supported firewall detected (firewalld or ufw)"
    echo "You may need to configure your firewall manually"
    exit 1
fi

echo "Detected firewall: $FIREWALL"
echo

# Configure firewall
if [ "$FIREWALL" = "firewalld" ]; then
    echo "Configuring firewalld..."

    # Add MultiShiva TCP port
    echo "1. Adding TCP port 53421 for MultiShiva..."
    firewall-cmd --permanent --add-port=53421/tcp

    # Add mDNS for auto-discovery
    echo "2. Adding mDNS (UDP port 5353) for auto-discovery..."
    if firewall-cmd --get-services | grep -q mdns; then
        firewall-cmd --permanent --add-service=mdns
    else
        firewall-cmd --permanent --add-port=5353/udp
    fi

    # Reload firewall
    echo "3. Reloading firewall..."
    firewall-cmd --reload

    echo
    echo "✅ Firewall configured successfully!"
    echo
    echo "Current configuration:"
    firewall-cmd --list-ports
    firewall-cmd --list-services

elif [ "$FIREWALL" = "ufw" ]; then
    echo "Configuring ufw..."

    # Add MultiShiva TCP port
    echo "1. Adding TCP port 53421 for MultiShiva..."
    ufw allow 53421/tcp comment 'MultiShiva'

    # Add mDNS for auto-discovery
    echo "2. Adding mDNS (UDP port 5353) for auto-discovery..."
    ufw allow 5353/udp comment 'mDNS'

    # Reload firewall
    echo "3. Reloading firewall..."
    ufw reload

    echo
    echo "✅ Firewall configured successfully!"
    echo
    echo "Current configuration:"
    ufw status
fi

echo
echo "🚀 MultiShiva is now configured!"
echo
echo "Next steps:"
echo "1. Start host: cargo run -- -m host"
echo "2. Start agent: cargo run -- -m agent"
echo "   (auto-discovery should now work)"
echo
