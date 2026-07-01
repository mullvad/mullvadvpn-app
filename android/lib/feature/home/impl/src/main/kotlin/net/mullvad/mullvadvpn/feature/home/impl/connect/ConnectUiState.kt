package net.mullvad.mullvadvpn.feature.home.impl.connect

import net.mullvad.mullvadvpn.lib.model.ConnectionPath
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.TunnelState

data class ConnectUiState(
    val internetLocation: GeoIpLocation?,
    val selectedRelayItemTitle: String?,
    val tunnelState: TunnelState,
    val hops: ConnectionPath,
    val locations: List<LatLong>,
    val inAppNotification: InAppNotification?,
    val deviceName: String?,
    val daysLeftUntilExpiry: Long?,
    val isPlayBuild: Boolean,
) {

    val showLoading =
        tunnelState is TunnelState.Connecting || tunnelState is TunnelState.Disconnecting

    companion object {
        val INITIAL =
            ConnectUiState(
                internetLocation = null,
                selectedRelayItemTitle = null,
                tunnelState = TunnelState.Disconnected(),
                hops = ConnectionPath(),
                locations = emptyList(),
                inAppNotification = null,
                deviceName = null,
                daysLeftUntilExpiry = null,
                isPlayBuild = false,
            )
    }
}
