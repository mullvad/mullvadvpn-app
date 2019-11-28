package net.mullvad.talpid.tunnel

sealed class BlockReason {
    class AuthFailed(val reason: String?) : BlockReason()
    class Ipv6Unavailable : BlockReason()
    class SetFirewallPolicyError : BlockReason()
    class SetDnsError : BlockReason()
    class StartTunnelError : BlockReason()
    class TunnelParameterError(val error: ParameterGenerationError) : BlockReason()
    class IsOffline : BlockReason()
    class TapAdapterProblem : BlockReason()
}
