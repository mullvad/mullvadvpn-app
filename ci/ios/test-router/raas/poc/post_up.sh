#!/usr/bin/env sh

CLIENT_IP="$1"
IFACE="$2"

echo client IP - "$CLIENT_IP"
echo interface - "$IFACE"

echo "Enabling port forwarding, will prompt for sudo password"
sysctl -w net.inet.ip.forwarding=1

# Derive local subnet from wireless interface
WIRELESS_IP=$(ifconfig -f address:cidr en0 inet | awk '/inet/{print $2}')
CIDR_NET="${WIRELESS_IP%.*}.0/24"

echo "Applying forwarding rules for $IFACE, routing $CLIENT_IP via $CIDR_NET"
pfctl -F all
sed "s/utun[0-9][0-9]*/$IFACE/g" ./poc/pf.utun4.conf \
  | sed "s|192.168.91.0/24|${CIDR_NET}|" \
  | sed "s|192.168.91.84|${CLIENT_IP}|" \
  | tee ./pf-rules \
  | pfctl -f -
pfctl -e || true
