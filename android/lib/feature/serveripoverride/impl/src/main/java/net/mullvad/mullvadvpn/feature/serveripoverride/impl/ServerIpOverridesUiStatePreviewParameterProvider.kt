package net.mullvad.mullvadvpn.feature.serveripoverride.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc

class ServerIpOverridesUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, ServerIpOverridesUiState>> {
    override val values =
        sequenceOf(
            ServerIpOverridesUiState(overridesActive = true).toLc(),
            ServerIpOverridesUiState(overridesActive = false).toLc(),
            Lc.Loading(true),
        )
}
