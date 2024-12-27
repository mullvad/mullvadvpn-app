package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.repository.InAppNotification

data class ConnectUiState(
    val location: GeoIpLocation?,
    val relayLocations: List<RelayItem.Location.City> = emptyList(),
    val selectedRelayItemTitle: String?,
    val selectedGeoLocationId: List<GeoLocationId>,
    val tunnelState: TunnelState,
    val showLocation: Boolean,
    val inAppNotification: InAppNotification?,
    val deviceName: String?,
    val daysLeftUntilExpiry: Int?,
    val isPlayBuild: Boolean,
) {

    val showLoading =
        tunnelState is TunnelState.Connecting || tunnelState is TunnelState.Disconnecting

    companion object {
        val INITIAL =
            ConnectUiState(
                location = null,
                selectedRelayItemTitle = null,
                selectedGeoLocationId = emptyList(),
                tunnelState = TunnelState.Disconnected(),
                showLocation = false,
                inAppNotification = null,
                deviceName = null,
                daysLeftUntilExpiry = null,
                isPlayBuild = false,
            )
    }
}
