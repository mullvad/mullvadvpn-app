package net.mullvad.mullvadvpn.model

sealed class BlockReason {
    class AuthFailed(val reason: String?) : BlockReason()
    class Ipv6Unavailable : BlockReason()
    class SetFirewallPolicyError : BlockReason()
    class SetDnsError : BlockReason()
    class StartTunnelError : BlockReason()
    class ParameterGeneration(val error: ParameterGenerationError) : BlockReason()
    class IsOffline : BlockReason()
    class TapAdapterProblem : BlockReason()
}

sealed class ParameterGenerationError {
    class NoMatchingRelay : ParameterGenerationError()
    class NoMatchingBridgeRelay : ParameterGenerationError()
    class NoWireguardKey : ParameterGenerationError()
    class CustomTunnelHostResultionError : ParameterGenerationError()
}
