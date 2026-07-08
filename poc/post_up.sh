#!/usr/bin/env sh

echo "Invalidating sudo to avoid accidental usage"
sudo -k

echo "Enabling port forwarding, will prompt for sudo password"
sudo sysctl -w net.inet.ip.forwarding=1

# Prompt for interface with visual pre-fill
read -e -i "utun4" -p "Interface name: " IFACE

echo "Bringing $IFACE up, requires sudo"
sudo ifconfig "$IFACE" 10.0.0.1 10.0.0.2 up

# Derive local subnet from wireless interface
WIRELESS_IP=$(ifconfig -f address:cidr en0 inet | awk '/inet/{print $2}')
CIDR_NET="${WIRELESS_IP%.*}.0/24"

# Prompt for client IP with visual pre-fill
read -e -i "${WIRELESS_IP%.*}." -p "Client device IP: " CLIENT_IP

echo "Applying forwarding rules for $IFACE, routing $CLIENT_IP via $CIDR_NET"
sudo pfctl -F all
sed "s/utun[0-9][0-9]*/$IFACE/g" pf.utun4.conf \
  | sed "s|192.168.91.0/24|${CIDR_NET}|" \
  | sed "s|192.168.91.84|${CLIENT_IP}|" \
  | sudo pfctl -f -
sudo pfctl -e

echo "Configuration done, dropping sudo"

./target/release/raas "$@"
sudo -k

