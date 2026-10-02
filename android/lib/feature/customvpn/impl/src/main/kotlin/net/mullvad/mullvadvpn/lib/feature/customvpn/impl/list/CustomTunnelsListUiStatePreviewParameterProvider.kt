package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc

class CustomTunnelsListUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, CustomTunnelsListUiState>> {
    override val values: Sequence<Lc<Unit, CustomTunnelsListUiState>> =
        sequenceOf(
            // Loading state
            Lc.Loading(Unit),
            // Default state
            Lc.Content(
                CustomTunnelsListUiState(
                    customVpnConfigState = null,
                    tunnelStats = null,
                )
            ),
        )
}
