package net.mullvad.talpid.tunnel

import java.net.InetAddress

sealed class ErrorStateCause {
    class AuthFailed(val reason: String?) : ErrorStateCause()
    object Ipv6Unavailable : ErrorStateCause()
    object SetFirewallPolicyError : ErrorStateCause()
    object SetDnsError : ErrorStateCause()
    class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()
    object StartTunnelError : ErrorStateCause()
    class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()
    object IsOffline : ErrorStateCause()
    object VpnPermissionDenied : ErrorStateCause()
}
