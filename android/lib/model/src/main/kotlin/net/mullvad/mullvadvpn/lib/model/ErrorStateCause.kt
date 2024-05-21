package net.mullvad.mullvadvpn.lib.model

import java.net.InetAddress

sealed class ErrorStateCause {
    class AuthFailed(private val reason: String?) : ErrorStateCause() {
        fun isCausedByExpiredAccount(): Boolean {
            return reason == AUTH_FAILED_REASON_EXPIRED_ACCOUNT
        }

        companion object {
            private const val AUTH_FAILED_REASON_EXPIRED_ACCOUNT = "[EXPIRED_ACCOUNT]"
        }
    }

    data object Ipv6Unavailable : ErrorStateCause()

    data class SetFirewallPolicyError(val firewallPolicyError: FirewallPolicyError) :
        ErrorStateCause()

    data object SetDnsError : ErrorStateCause()

    data class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()

    data object StartTunnelError : ErrorStateCause()

    data class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()

    data object IsOffline : ErrorStateCause()

    data object VpnPermissionDenied : ErrorStateCause()
}
