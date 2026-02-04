package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
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
