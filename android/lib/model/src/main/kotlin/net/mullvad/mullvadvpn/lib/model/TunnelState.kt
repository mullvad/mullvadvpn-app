package net.mullvad.mullvadvpn.lib.model

sealed class TunnelState {
    data class Disconnected(val location: GeoIpLocation? = null) : TunnelState()

    data class Connecting(
        val endpoint: TunnelEndpoint?,
        val location: GeoIpLocation?,
        val featureIndicators: List<FeatureIndicator>,
    ) : TunnelState()

    data class Connected(
        val endpoint: TunnelEndpoint,
        val location: GeoIpLocation?,
        val featureIndicators: List<FeatureIndicator>,
    ) : TunnelState()

    data class Disconnecting(val actionAfterDisconnect: ActionAfterDisconnect) : TunnelState()

    data class Error(val errorState: ErrorState) : TunnelState()

    fun featureIndicators(): List<FeatureIndicator>? =
        when (this) {
            is Connected -> featureIndicators
            is Connecting -> featureIndicators
            else -> null
        }

    fun location(): GeoIpLocation? =
        when (this) {
            is Connected -> location
            is Connecting -> location
            is Disconnecting -> null
            is Disconnected -> location
            is Error -> null
        }

    fun isConnectingOrConnected(): Boolean =
        when (this) {
            is Connected,
            is Connecting -> true
            else -> false
        }

    fun isSecured(): Boolean =
        when (this) {
            is Connected,
            is Connecting,
            is Disconnecting -> true
            is Disconnected -> false
            is Error -> this.errorState.isBlocking
        }

    fun isBlocked(): Boolean =
        when (this) {
            is Connected,
            is Disconnected -> false
            is Connecting,
            is Disconnecting -> true
            is Error -> this.errorState.isBlocking
        }
}
