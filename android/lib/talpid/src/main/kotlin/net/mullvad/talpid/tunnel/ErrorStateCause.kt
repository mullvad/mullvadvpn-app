package net.mullvad.talpid.tunnel

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

private const val AUTH_FAILED_REASON_EXPIRED_ACCOUNT = "[EXPIRED_ACCOUNT]"

sealed class ErrorStateCause : Parcelable {
    @Parcelize
    class AuthFailed(private val reason: String?) : ErrorStateCause() {
        fun isCausedByExpiredAccount(): Boolean {
            return reason == AUTH_FAILED_REASON_EXPIRED_ACCOUNT
        }
    }

    @Parcelize data object Ipv6Unavailable : ErrorStateCause()

    @Parcelize
    data class SetFirewallPolicyError(val firewallPolicyError: FirewallPolicyError) :
        ErrorStateCause()

    @Parcelize data object SetDnsError : ErrorStateCause()

    @Parcelize
    data class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()

    @Parcelize data object StartTunnelError : ErrorStateCause()

    @Parcelize
    data class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()

    @Parcelize data object IsOffline : ErrorStateCause()

    @Parcelize data object VpnPermissionDenied : ErrorStateCause()
}
