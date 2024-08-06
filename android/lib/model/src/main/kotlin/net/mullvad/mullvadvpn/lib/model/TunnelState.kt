package net.mullvad.mullvadvpn.lib.model

sealed class TunnelState {
    data class Disconnected(val location: GeoIpLocation? = null, val age: Int = 0) : TunnelState()

    data class Connecting(val endpoint: TunnelEndpoint?, val location: GeoIpLocation?) :
        TunnelState()

    data class Connected(val endpoint: TunnelEndpoint, val location: GeoIpLocation?) :
        TunnelState()

    data class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) : TunnelState()

    data class Error(val errorState: ErrorState) : TunnelState()

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
