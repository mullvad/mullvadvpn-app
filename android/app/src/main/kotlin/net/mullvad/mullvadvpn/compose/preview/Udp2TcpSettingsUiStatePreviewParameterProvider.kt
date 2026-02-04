package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.Udp2TcpSettingsUiState
import net.mullvad.mullvadvpn.constant.UDP2TCP_PRESET_PORTS
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint

class Udp2TcpSettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, Udp2TcpSettingsUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            Udp2TcpSettingsUiState(port = Constraint.Any).toLc(),
            Udp2TcpSettingsUiState(port = Constraint.Only(UDP2TCP_PRESET_PORTS.first())).toLc(),
        )
}
