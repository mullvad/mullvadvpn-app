package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.InAppNotification

data class ConnectUiState(
    val location: GeoIpLocation?,
    val selectedRelayItemTitle: String?,
    val tunnelState: TunnelState,
    val showLocation: Boolean,
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
                location = null,
                selectedRelayItemTitle = null,
                tunnelState = TunnelState.Disconnected(),
                showLocation = false,
                inAppNotification = null,
                deviceName = null,
                daysLeftUntilExpiry = null,
                isPlayBuild = false,
            )
    }
}
