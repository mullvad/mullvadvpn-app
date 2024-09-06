package net.mullvad.mullvadvpn.lib.model

enum class FeatureIndicator {
    QUANTUM_RESISTANCE,
    SPLIT_TUNNELING,
    UDP_2_TCP,
    LAN_SHARING,
    DNS_CONTENT_BLOCKERS,
    CUSTOM_DNS,
    SERVER_IP_OVERRIDE,
    CUSTOM_MTU,
    DAITA,
    // Currently not supported
    // LOCKDOWN_MODE,
    // SHADOWSOCKS,
    // MULTIHOP,
    // BRIDGE_MODE,
    // CUSTOM_MSS_FIX,
    // UNRECOGNIZED,
}
