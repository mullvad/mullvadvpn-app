#!/usr/bin/env sh

SOURCE_IFACE="$1"
SINK_IFACE="$2"

echo source interface - "$SOURCE_IFACE"
echo sink interface - "$SINK_IFACE"

echo "Enabling port forwarding, will prompt for sudo password"
sysctl -w net.inet.ip.forwarding=1

# The interface holding the default route; clients reach us through it
WAN_IF=$(route -n get default | awk '/interface:/{print $2}')

# The peer address of the sink, needed as the reply-to gateway
SINK_PEER=$(ifconfig "$SINK_IFACE" inet | awk '/inet /{print $4}')

echo "Applying forwarding rules: $WAN_IF -> $SOURCE_IFACE, replies via $SINK_IFACE"
pfctl -F all
sed "s|source_utun|$SOURCE_IFACE|g" ./poc/pf.utun4.conf \
  | sed "s|sink_utun|${SINK_IFACE}|g" \
  | sed "s|sink_peer|${SINK_PEER}|g" \
  | sed "s|wan_if|${WAN_IF}|g" \
  | tee ./pf-rules \
  | pfctl -f - \
  || { echo "pfctl rejected the rules in ./pf-rules" >&2; exit 1; }
pfctl -e || true
