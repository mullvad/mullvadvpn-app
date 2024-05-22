package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.repository.InAppNotification

data class ConnectUiState(
    val location: GeoIpLocation?,
    val selectedRelayItemTitle: String?,
    val tunnelState: TunnelState,
    val inAddress: Triple<String, Int, TransportProtocol>?,
    val outAddress: String,
    val showLocation: Boolean,
    val inAppNotification: InAppNotification?,
    val deviceName: String?,
    val daysLeftUntilExpiry: Int?,
    val isPlayBuild: Boolean,
) {

    val showLocationInfo: Boolean =
        tunnelState !is TunnelState.Disconnected && location?.hostname != null
    val showLoading =
        tunnelState is TunnelState.Connecting || tunnelState is TunnelState.Disconnecting

    companion object {
        val INITIAL =
            ConnectUiState(
                location = null,
                selectedRelayItemTitle = null,
                tunnelState = TunnelState.Disconnected(),
                inAddress = null,
                outAddress = "",
                showLocation = false,
                inAppNotification = null,
                deviceName = null,
                daysLeftUntilExpiry = null,
                isPlayBuild = false,
            )
    }
}
