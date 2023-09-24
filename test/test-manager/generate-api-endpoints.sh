#!/usr/bin/env bash

# Output known API IPs and bridge relay IPs

dig +short api.mullvad.net
dig +short api.devmole.eu
dig +short api.stagemole.eu
echo "45.83.223.196" # old prod
curl -s https://api.mullvad.net/app/v1/relays | jq -r '.bridge.relays[] | .ipv4_addr_in'
