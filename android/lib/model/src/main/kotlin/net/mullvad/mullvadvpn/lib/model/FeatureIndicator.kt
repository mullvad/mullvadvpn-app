package net.mullvad.mullvadvpn.lib.model

// The order of the variants match the priority order and can be sorted on.
enum class FeatureIndicator {
    // All anti-censorship feature indicators needs to be grouped together for animations to work as
    // expected.
    // This is derived from the other feature indicators, it is never sent by the daemon
    ANTI_CENSORSHIP,
    UDP_2_TCP,
    SHADOWSOCKS,
    QUIC,
    LWO,
    WIREGUARD_PORT,
    DAITA,
    MULTIHOP,
    MULTIHOP_AUTO,
    SPLIT_TUNNELING,
    LAN_SHARING,
    DNS_CONTENT_BLOCKERS,
    CUSTOM_DNS,
    SERVER_IP_OVERRIDE,
    CUSTOM_MTU,
    QUANTUM_RESISTANCE,
    CUSTOM_VPN,
}
