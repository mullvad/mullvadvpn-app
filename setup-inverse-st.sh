#!/usr/bin/env bash

set -e

namespace="mullvad-ns"
tun_iface="wg0-mullvad"

echo "Step 0: Configure DNS"
mkdir -p /etc/netns/$namespace/
echo "nameserver 10.64.0.1" > /etc/netns/$namespace/resolv.conf

echo "hosts: files dns" > /etc/netns/$namespace/nsswitch.conf

echo "Step 1: Recreating $namespace namespace"
ip netns delete $namespace || true
ip netns add $namespace || true

echo "Step 2: Firewall stuff"
ip netns exec $namespace nft -f - <<EOF
table inet filter {
    chain output {
        type filter hook output priority 0; policy accept;
        ip daddr 10.64.0.1 udp dport 53 accept
        ip daddr 10.64.0.1 tcp dport 53 accept
        udp dport 53 drop
        tcp dport 53 drop
    }
}
EOF


tunnel_ip=$(ip addr show $tun_iface | grep -oP '(?<=inet\s)\d+(\.\d+){3}/\d+')
echo "Tunnel IP: $tunnel_ip"

echo "Step 3: Move $tun_iface to $namespace namespace"

ip link set $tun_iface netns $namespace

echo "Step 4: Configuring tun interface"

echo "Configuring IP for $tun_iface"
ip -n $namespace link set dev lo up
ip -n $namespace link set $tun_iface up
ip -n $namespace addr add dev $tun_iface $tunnel_ip

echo "Add default route for $tun_iface"
ip -n $namespace route add default dev $tun_iface

echo "Performing various incantations"
echo "Making things very secure"

echo "Success."
