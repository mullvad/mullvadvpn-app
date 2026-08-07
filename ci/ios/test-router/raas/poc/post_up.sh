#!/usr/bin/env sh

CLIENT_IP="$1"
SOURCE_IFACE="$2"
SINK_IFACE="$3"

echo client IP - "$CLIENT_IP"
echo source interface - "$SOURCE_IFACE"
echo sink interface - "$SINK_IFACE"

echo "Enabling port forwarding, will prompt for sudo password"
sysctl -w net.inet.ip.forwarding=1

# Derive local subnet from wireless interface
WIRELESS_IP=$(ifconfig -f address:cidr en0 inet | awk '/inet/{print $2}')
CIDR_NET="${WIRELESS_IP%.*}.0/24"

# The peer address of the sink, needed as the reply-to gateway
SINK_PEER=$(ifconfig "$SINK_IFACE" inet | awk '/inet /{print $4}')

echo "Applying forwarding rules for $SOURCE_IFACE, routing $CLIENT_IP via $CIDR_NET"
pfctl -F all
sed "s|source_utun|$SOURCE_IFACE|g" ./poc/pf.utun4.conf \
  | sed "s|client_ip|${CLIENT_IP}|g" \
  | sed "s|sink_utun|${SINK_IFACE}|g" \
  | sed "s|sink_peer|${SINK_PEER}|g" \
  | tee ./pf-rules \
  | pfctl -f - \
  || { echo "pfctl rejected the rules in ./pf-rules" >&2; exit 1; }
pfctl -e || true
