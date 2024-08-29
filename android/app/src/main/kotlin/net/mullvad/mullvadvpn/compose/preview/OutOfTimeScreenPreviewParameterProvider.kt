package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateConnectingState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateDisconnectedState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateErrorState
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState

class OutOfTimeScreenPreviewParameterProvider : PreviewParameterProvider<OutOfTimeUiState> {
    override val values: Sequence<OutOfTimeUiState> =
        sequenceOf(
            OutOfTimeUiState(
                tunnelState = generateDisconnectedState(),
                "Heroic Frog",
                showSitePayment = true,
            ),
            OutOfTimeUiState(
                tunnelState =
                    generateConnectingState(featureIndicators = 0, quantumResistant = false),
                "Strong Rabbit",
                showSitePayment = true,
            ),
            OutOfTimeUiState(
                tunnelState = generateErrorState(isBlocking = true),
                deviceName = "Stable Horse",
                showSitePayment = true,
            ),
        )
}
