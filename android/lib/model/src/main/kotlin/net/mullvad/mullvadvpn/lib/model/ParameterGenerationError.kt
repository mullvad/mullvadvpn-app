package net.mullvad.mullvadvpn.lib.model

sealed interface ParameterGenerationError {
    data class NoMatchingRelayEntry(val unavailableIpVersion: Boolean) : ParameterGenerationError

    data object NoMatchingRelayExit : ParameterGenerationError

    data class NoMatchingRelay(val unavailableIpVersion: Boolean) : ParameterGenerationError

    data object NoMatchingBridgeRelay : ParameterGenerationError

    data object CustomTunnelHostResolutionError : ParameterGenerationError

    data object Ipv4_Unavailable : ParameterGenerationError

    data object Ipv6_Unavailable : ParameterGenerationError
}
