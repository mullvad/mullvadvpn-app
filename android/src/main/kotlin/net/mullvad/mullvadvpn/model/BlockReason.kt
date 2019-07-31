package net.mullvad.mullvadvpn.model

sealed class BlockReason {
    class AuthFailed(val reason: String?) : BlockReason()
    class Ipv6Unavailable : BlockReason()
    class SetFirewallPolicyError : BlockReason()
    class SetDnsError : BlockReason()
    class StartTunnelError : BlockReason()
    class NoMatchingRelay : BlockReason()
    class IsOffline : BlockReason()
    class TapAdapterProblem : BlockReason()
}
