package net.mullvad.mullvadvpn.lib.model

enum class ParameterGenerationError {
    NoMatchingRelayEntry,
    NoMatchingRelayExit,
    NoMatchingRelay,
    NoMatchingBridgeRelay,
    CustomTunnelHostResolutionError,
    Ipv4Unavailable,
    Ipv6Unavailable,
    NoMatchingRelayEntryUnavailableIpVersion,
    NoMatchingRelayUnavailableIpVersion,
}
