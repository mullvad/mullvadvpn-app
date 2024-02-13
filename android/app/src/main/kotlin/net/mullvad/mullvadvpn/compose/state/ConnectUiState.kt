package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.talpid.net.TransportProtocol

data class ConnectUiState(
    val location: GeoIpLocation?,
    val selectedRelayItem: RelayItem?,
    val tunnelUiState: TunnelState,
    val tunnelRealState: TunnelState,
    val inAddress: Triple<String, Int, TransportProtocol>?,
    val outAddress: String,
    val showLocation: Boolean,
    val inAppNotification: InAppNotification?,
    val deviceName: String?,
    val daysLeftUntilExpiry: Int?,
    val isPlayBuild: Boolean,
    val animateMap: Boolean
) {

    val showLocationInfo: Boolean =
        tunnelRealState !is TunnelState.Disconnected && location?.hostname != null
    val showLoading =
        tunnelRealState is TunnelState.Connecting || tunnelRealState is TunnelState.Disconnecting

    companion object {
        val INITIAL =
            ConnectUiState(
                location = null,
                selectedRelayItem = null,
                tunnelUiState = TunnelState.Disconnected(),
                tunnelRealState = TunnelState.Disconnected(),
                inAddress = null,
                outAddress = "",
                showLocation = false,
                inAppNotification = null,
                deviceName = null,
                daysLeftUntilExpiry = null,
                isPlayBuild = false,
                animateMap = true
            )
    }
}
