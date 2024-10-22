# Apple notes

The first packet is always dropped when a connection is routed and NATed


The NAT rules do not match up with the firewall rules in regards to the relay


```
# NAT-rule
no nat inet from any to 185.213.154.68

# FW-rule
pass out quick inet proto udp from any to 185.213.154.68 port = 49020 user = 0 keep state
```

