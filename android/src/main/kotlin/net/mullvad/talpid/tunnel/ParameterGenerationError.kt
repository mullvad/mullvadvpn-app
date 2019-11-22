package net.mullvad.talpid.tunnel

enum class ParameterGenerationError {
    NoMatchingRelay, NoMatchingBridgeRelay, NoWireguardKey, CustomTunnelHostResultionError
}
