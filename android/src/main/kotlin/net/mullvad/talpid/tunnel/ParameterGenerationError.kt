package net.mullvad.talpid.tunnel

sealed class ParameterGenerationError {
    class NoMatchingRelay : ParameterGenerationError()
    class NoMatchingBridgeRelay : ParameterGenerationError()
    class NoWireguardKey : ParameterGenerationError()
    class CustomTunnelHostResultionError : ParameterGenerationError()
}
