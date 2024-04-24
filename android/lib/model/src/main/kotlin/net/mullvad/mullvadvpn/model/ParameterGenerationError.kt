package net.mullvad.mullvadvpn.model

enum class ParameterGenerationError {
    NoMatchingRelay,
    NoMatchingBridgeRelay,
    NoWireguardKey,
    CustomTunnelHostResultionError
}
