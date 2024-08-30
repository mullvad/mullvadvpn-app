#!/usr/bin/env bash

set -e

namespace="mullvad-ns-exclude"
tun_iface="wg0-mullvad"

default_ns_iface=vethmole0
exclude_ns_iface=vethmole1

default_ns_net=172.25.1.1/30
exclude_ns_net=172.25.1.2/30
exclude_ns_gateway=172.25.1.1

# TODO: Use original host config, if possible
echo "Configure DNS"
mkdir -p /etc/netns/$namespace/
echo "nameserver 1.1.1.1" > /etc/netns/$namespace/resolv.conf
echo "hosts: files dns" > /etc/netns/$namespace/nsswitch.conf

echo "Recreating namespace $namespace"
ip netns delete $namespace || true
ip netns add $namespace || true

echo "Creating veth pair"
ip link del dev $default_ns_iface || true
ip link add dev $default_ns_iface type veth peer name $exclude_ns_iface

echo "Setting up default namespace veth interface $default_ns_iface"
ip addr add $default_ns_net dev $default_ns_iface
ip link set dev $default_ns_iface up

echo "Moving $exclude_ns_iface to namespace $namespace"
ip link set dev $exclude_ns_iface netns $namespace

echo "Configuring $exclude_ns_iface"
ip -n $namespace addr add $exclude_ns_net dev $exclude_ns_iface
ip -n $namespace link set dev lo up
ip -n $namespace link set dev $exclude_ns_iface up

echo "Add default route for $exclude_ns_iface"
ip -n $namespace link set dev $exclude_ns_iface up
ip -n $namespace route add default via $exclude_ns_gateway

echo "Set up forwarding"

# TODO: only for veth pair
sysctl net.ipv4.conf.all.forwarding=1

nft delete table inet exclude_nat_test >/dev/null || true
nft delete table inet exclude_filter_test >/dev/null || true
nft -f - <<EOF
table inet exclude_nat_test {
    chain prerouting {
        type nat hook prerouting priority mangle; policy accept;
        # TODO: routing or nft?
        #ip daddr 10.64.0.1 counter accept
        ip saddr $default_ns_net ct mark set 0x6d6f6c65
        ip saddr $default_ns_net meta mark set ct mark
    }
    chain postrouting {
        type nat hook postrouting priority 100; policy accept;
        # TODO: != wg tun
        ip saddr $default_ns_net masquerade
    }
}
table inet exclude_filter_test {
    chain forward {
        type filter hook forward priority 0; policy accept;
        iifname "$default_ns_iface" oifname != "$default_ns_iface" accept
        oifname "$default_ns_iface" iifname != "$default_ns_iface" accept
    }
}
EOF

# TODO: nft or routing?
echo "Set up routing"
ip rule del from $default_ns_net table main || true
ip rule add from $default_ns_net table main
