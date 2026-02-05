package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AntiCensorshipSettingsUiState
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode

class AntiCensorshipUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, AntiCensorshipSettingsUiState>> {
    override val values =
        sequenceOf(
            AntiCensorshipSettingsUiState.from(
                    isModal = false,
                    selectedWireguardPort = Constraint.Any,
                    obfuscationMode = ObfuscationMode.Udp2Tcp,
                    selectedUdp2TcpObfuscationPort = Constraint.Any,
                    selectedShadowsocksObfuscationPort = Constraint.Any,
                )
                .toLc(),
            Lc.Loading(true),
        )
}
