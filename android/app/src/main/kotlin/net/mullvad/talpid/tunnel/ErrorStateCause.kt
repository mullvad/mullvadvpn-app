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

    @Parcelize
    object Ipv6Unavailable : ErrorStateCause()

    @Parcelize
    object SetFirewallPolicyError : ErrorStateCause()

    @Parcelize
    object SetDnsError : ErrorStateCause()

    @Parcelize
    class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause()

    @Parcelize
    object StartTunnelError : ErrorStateCause()

    @Parcelize
    class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause()

    @Parcelize
    object IsOffline : ErrorStateCause()

    @Parcelize
    object VpnPermissionDenied : ErrorStateCause()
}
