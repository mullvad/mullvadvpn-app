package net.mullvad.mullvadvpn.feature.home.impl.outoftime

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.feature.home.impl.TunnelStatePreviewData.generateConnectingState
import net.mullvad.mullvadvpn.feature.home.impl.TunnelStatePreviewData.generateDisconnectedState
import net.mullvad.mullvadvpn.feature.home.impl.TunnelStatePreviewData.generateErrorState
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc

class OutOfTimeScreenPreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, OutOfTimeUiState>> {
    override val values: Sequence<Lc<Unit, OutOfTimeUiState>> =
        sequenceOf(
            OutOfTimeUiState(
                    tunnelState = generateDisconnectedState(),
                    "Heroic Frog",
                    showSitePayment = true,
                )
                .toLc(),
            OutOfTimeUiState(
                    tunnelState =
                        generateConnectingState(featureIndicators = 0, quantumResistant = false),
                    "Strong Rabbit",
                    showSitePayment = true,
                )
                .toLc(),
            OutOfTimeUiState(
                    tunnelState = generateErrorState(isBlocking = true),
                    deviceName = "Stable Horse",
                    showSitePayment = true,
                )
                .toLc(),
            Lc.Loading(Unit),
        )
}
