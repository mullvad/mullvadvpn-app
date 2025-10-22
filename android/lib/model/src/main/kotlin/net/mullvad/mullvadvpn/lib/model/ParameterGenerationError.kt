package net.mullvad.mullvadvpn.lib.model

enum class ParameterGenerationError {
    NoMatchingRelayEntry,
    NoMatchingRelayExit,
    NoMatchingRelay,
    NoMatchingBridgeRelay,
    NoWireguardKey,
    CustomTunnelHostResolutionError,
    Ipv4_Unavailable,
    Ipv6_Unavailable,
}
