#!/usr/bin/env bash

# This script generates bindings for certain pcap and pktap symbols.
# bindgen is required: cargo install bindgen-cli

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

curl https://opensource.apple.com/source/xnu/xnu-3789.41.3/bsd/net/pktap.h -o include/pktap.h
curl https://opensource.apple.com/source/libpcap/libpcap-67/libpcap/pcap/pcap.h -o include/pcap.h
curl https://opensource.apple.com/source/xnu/xnu-3789.41.3/bsd/net/bpf.h -o include/bpf.h

bindgen "include/bindings.h" -o ./bindings.rs \
    --allowlist-item "^pcap_create" \
    --allowlist-item "^pcap_set_want_pktap" \
    --allowlist-item "^pktap_header" \
    --allowlist-item "PCAP_ERRBUF_SIZE" \
    --allowlist-item "^BIOCSWANTPKTAP" \
    --allowlist-item "^bpf_stat"
