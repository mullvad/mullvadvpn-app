package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SelectPortUiState
import net.mullvad.mullvadvpn.constant.UDP2TCP_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class SelectPortUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, SelectPortUiState>> {
    override val values: Sequence<Lc<Unit, SelectPortUiState>> =
        sequenceOf(
            SelectPortUiState(
                    portType = PortType.Udp2Tcp,
                    presetPorts = UDP2TCP_PRESET_PORTS,
                    customPortEnabled = false,
                    title = "Select port",
                )
                .toLc(),
            SelectPortUiState(
                    portType = PortType.Lwo,
                    port = Constraint.Only(Port(1)),
                    presetPorts = emptyList(),
                    customPortEnabled = true,
                    title = "Select port",
                )
                .toLc(),
        )
}
