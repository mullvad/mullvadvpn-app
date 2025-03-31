package net.mullvad.mullvadvpn.lib.model

// The order of the variants match the priority order and can be sorted on.
enum class FeatureIndicator {
    DAITA,
    DAITA_MULTIHOP,
    QUANTUM_RESISTANCE,
    MULTIHOP,
    SPLIT_TUNNELING,
    UDP_2_TCP,
    SHADOWSOCKS,
    LAN_SHARING,
    DNS_CONTENT_BLOCKERS,
    CUSTOM_DNS,
    SERVER_IP_OVERRIDE,
    CUSTOM_MTU,
}
