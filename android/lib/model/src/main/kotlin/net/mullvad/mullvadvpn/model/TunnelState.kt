package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.TunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState

sealed class TunnelState : Parcelable {
    @Parcelize
    data class Disconnected(val location: GeoIpLocation? = null) : TunnelState(), Parcelable

    @Parcelize
    class Connecting(val endpoint: TunnelEndpoint?, val location: GeoIpLocation?) :
        TunnelState(), Parcelable

    @Parcelize
    class Connected(val endpoint: TunnelEndpoint, val location: GeoIpLocation?) :
        TunnelState(), Parcelable

    @Parcelize
    class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) :
        TunnelState(), Parcelable

    @Parcelize class Error(val errorState: ErrorState) : TunnelState(), Parcelable

    fun location(): GeoIpLocation? {
        return when (this) {
            is Connected -> location
            is Connecting -> location
            is Disconnecting -> null
            is Disconnected -> location
            is Error -> null
        }
    }

    fun isSecured(): Boolean {
        return when (this) {
            is Connected,
            is Connecting,
            is Disconnecting, -> true
            is Disconnected -> false
            is Error -> this.errorState.isBlocking
        }
    }
}
