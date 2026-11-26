package net.mullvad.mullvadvpn.lib.model

// The order of the variants match the priority order and can be sorted on.
enum class FeatureIndicator {
    // All anti-censorship feature indicators needs to be grouped together for animations to work as
    // expected.
    // ANTI_CENSORSHIP is derived from the other feature indicators, it is never sent by the daemon.
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

val ANTI_CENSORSHIP_FEATURE_INDICATORS =
    listOf(
        FeatureIndicator.ANTI_CENSORSHIP,
        FeatureIndicator.WIREGUARD_PORT,
        FeatureIndicator.LWO,
        FeatureIndicator.QUIC,
        FeatureIndicator.SHADOWSOCKS,
        FeatureIndicator.UDP_2_TCP,
    )
