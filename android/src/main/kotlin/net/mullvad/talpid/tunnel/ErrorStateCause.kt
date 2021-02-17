package net.mullvad.talpid.tunnel

import java.net.InetAddress

sealed class ErrorStateCause {
    class AuthFailed(val reason: String?) : ErrorStateCause()

    class Ipv6Unavailable : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = Ipv6Unavailable()
        }
    }

    class SetFirewallPolicyError : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = SetFirewallPolicyError()
        }
    }

    class SetDnsError : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = SetDnsError()
        }
    }

    class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()

    class StartTunnelError : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = StartTunnelError()
        }
    }

    class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()

    class IsOffline : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = IsOffline()
        }
    }

    class VpnPermissionDenied : ErrorStateCause() {
        companion object {
            @JvmStatic
            val INSTANCE = VpnPermissionDenied()
        }
    }
}
