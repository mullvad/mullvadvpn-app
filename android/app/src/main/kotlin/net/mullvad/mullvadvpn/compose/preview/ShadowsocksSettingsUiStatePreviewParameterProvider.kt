package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsUiState
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port

class ShadowsocksSettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, ShadowsocksSettingsUiState>> {
    override val values: Sequence<Lc<Unit, ShadowsocksSettingsUiState>> =
        sequenceOf(
            Lc.Loading(Unit),
            ShadowsocksSettingsUiState(port = Constraint.Any).toLc(),
            ShadowsocksSettingsUiState(port = Constraint.Only(Port(1)), customPort = Port(1)).toLc(),
        )
}
