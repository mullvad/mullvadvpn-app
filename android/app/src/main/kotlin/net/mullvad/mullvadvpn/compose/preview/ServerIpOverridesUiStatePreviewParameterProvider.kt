package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiState

class ServerIpOverridesUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, ServerIpOverridesUiState>> {
    override val values =
        sequenceOf(
            ServerIpOverridesUiState(overridesActive = true).toLc(),
            ServerIpOverridesUiState(overridesActive = false).toLc(),
            Lc.Loading(true),
        )
}
