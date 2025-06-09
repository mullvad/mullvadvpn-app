package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class WelcomeScreenUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, WelcomeUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            WelcomeUiState(
                    tunnelState = TunnelStatePreviewData.generateDisconnectedState(),
                    accountNumber = AccountNumber("4444555566667777"),
                    deviceName = "Happy Mole",
                    showSitePayment = false,
                )
                .toLc(),
            WelcomeUiState(
                    tunnelState =
                        TunnelStatePreviewData.generateConnectedState(featureIndicators = 1, false),
                    accountNumber = AccountNumber("4444555566667777"),
                    deviceName = "Happy Mole",
                    showSitePayment = true,
                )
                .toLc(),
        )
}
