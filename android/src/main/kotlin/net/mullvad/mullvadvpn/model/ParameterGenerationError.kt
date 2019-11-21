package net.mullvad.mullvadvpn.model

sealed class ParameterGenerationError {
    class NoMatchingRelay : ParameterGenerationError()
    class NoMatchingBridgeRelay : ParameterGenerationError()
    class NoWireguardKey : ParameterGenerationError()
    class CustomTunnelHostResultionError : ParameterGenerationError()
}
