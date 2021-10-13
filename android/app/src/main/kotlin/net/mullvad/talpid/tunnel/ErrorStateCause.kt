package net.mullvad.talpid.tunnel

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

sealed class ErrorStateCause : Parcelable {
    @Parcelize
    class AuthFailed(val reason: String?) : ErrorStateCause()

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
