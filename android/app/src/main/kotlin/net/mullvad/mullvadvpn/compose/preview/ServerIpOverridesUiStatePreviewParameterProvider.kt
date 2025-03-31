package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiState

class ServerIpOverridesUiStatePreviewParameterProvider :
    PreviewParameterProvider<ServerIpOverridesUiState> {
    override val values =
        sequenceOf(
            ServerIpOverridesUiState.Loaded(overridesActive = true),
            ServerIpOverridesUiState.Loaded(overridesActive = false),
            ServerIpOverridesUiState.Loading(),
        )
}
