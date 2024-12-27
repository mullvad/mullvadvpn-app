package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import java.net.InetAddress
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation

class ConnectUiStatePreviewParameterProvider : PreviewParameterProvider<ConnectUiState> {
    override val values = sequenceOf(ConnectUiState.INITIAL) + generateOtherStates()
}

private fun generateOtherStates(): Sequence<ConnectUiState> =
    sequenceOf(
            TunnelStatePreviewData.generateConnectedState(
                featureIndicators = 8,
                quantumResistant = true,
            ),
            TunnelStatePreviewData.generateDisconnectedState(),
            TunnelStatePreviewData.generateConnectingState(
                featureIndicators = 4,
                quantumResistant = false,
            ),
            TunnelStatePreviewData.generateDisconnectingState(
                actionAfterDisconnect = ActionAfterDisconnect.Reconnect
            ),
            TunnelStatePreviewData.generateErrorState(isBlocking = true),
        )
        .map { state ->
            ConnectUiState(
                location =
                    GeoIpLocation(
                        ipv4 = InetAddress.getLocalHost(),
                        ipv6 = null,
                        country = "Sweden",
                        city = "Göteborg",
                        latitude = 23.3,
                        longitude = 12.99,
                        hostname = "Hostname",
                        entryHostname = "EntryHostname",
                    ),
                selectedRelayItemTitle = "Relay Title",
                tunnelState = state,
                showLocation = true,
                inAppNotification = null,
                deviceName = "Cool Beans",
                daysLeftUntilExpiry = 42,
                selectedGeoLocationId = null,
                isPlayBuild = true,
            )
        }
