package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateConnectedState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateConnectingState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateDisconnectedState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateDisconnectingState
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewData.generateErrorState
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.TunnelState

class TunnelStatePreviewParameterProvider : PreviewParameterProvider<TunnelState> {
    override val values: Sequence<TunnelState> =
        sequenceOf(
            generateDisconnectedState(),
            generateConnectingState(featureIndicators = 0, quantumResistant = false),
            generateConnectingState(featureIndicators = 0, quantumResistant = true),
            generateConnectedState(featureIndicators = 0, quantumResistant = false),
            generateConnectedState(featureIndicators = 0, quantumResistant = true),
            generateDisconnectingState(actionAfterDisconnect = ActionAfterDisconnect.Block),
            generateDisconnectingState(actionAfterDisconnect = ActionAfterDisconnect.Nothing),
            generateDisconnectingState(actionAfterDisconnect = ActionAfterDisconnect.Reconnect),
            generateErrorState(isBlocking = true),
            generateErrorState(isBlocking = false)
        )
}
