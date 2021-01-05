package net.mullvad.talpid.tunnel

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
sealed class ErrorStateCause : Parcelable {
    @Parcelize
    class AuthFailed(val reason: String?) : ErrorStateCause(), Parcelable

    @Parcelize
    class Ipv6Unavailable : ErrorStateCause(), Parcelable

    @Parcelize
    class SetFirewallPolicyError : ErrorStateCause(), Parcelable

    @Parcelize
    class SetDnsError : ErrorStateCause(), Parcelable

    @Parcelize
    class InvalidDnsServers(val addresses: ArrayList<InetAddress>) : ErrorStateCause(), Parcelable

    @Parcelize
    class StartTunnelError : ErrorStateCause(), Parcelable

    @Parcelize
    class TunnelParameterError(val error: ParameterGenerationError) : ErrorStateCause(), Parcelable

    @Parcelize
    class IsOffline : ErrorStateCause(), Parcelable

    @Parcelize
    class VpnPermissionDenied : ErrorStateCause(), Parcelable
}
