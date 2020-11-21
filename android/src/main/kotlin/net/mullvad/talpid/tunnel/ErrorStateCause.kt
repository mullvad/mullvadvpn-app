package net.mullvad.talpid.tunnel

import java.net.InetAddress

sealed class ErrorStateCause {
    class AuthFailed(val reason: String?) : ErrorStateCause()
    class Ipv6Unavailable : ErrorStateCause()
    class SetFirewallPolicyError : ErrorStateCause()
    class SetDnsError : ErrorStateCause()
    class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()
    class StartTunnelError : ErrorStateCause()
    class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()
    class IsOffline : ErrorStateCause()
    class VpnPermissionDenied : ErrorStateCause()
}
