package net.mullvad.mullvadvpn.lib.model

enum class ParameterGenerationError {
    NoMatchingRelay,
    NoMatchingBridgeRelay,
    NoWireguardKey,
    CustomTunnelHostResolutionError,
    IpVersionUnavailable,
}
