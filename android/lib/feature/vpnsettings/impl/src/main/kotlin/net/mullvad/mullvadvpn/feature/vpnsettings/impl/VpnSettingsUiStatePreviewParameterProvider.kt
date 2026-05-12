package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

private const val MTU = 1337

class VpnSettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, VpnSettingsUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(true),
            VpnSettingsUiState.from(
                    mtu = Mtu(MTU),
                    isLocalNetworkSharingEnabled = true,
                    quantumResistant = QuantumResistantState.On,
                    systemVpnSettingsAvailable = true,
                    autoStartAndConnectOnBoot = true,
                    isIpv6Enabled = true,
                    obfuscationMode = ObfuscationMode.Udp2Tcp,
                    deviceIpVersion = Constraint.Any,
                    isModal = false,
                )
                .toLc(),
        )
}
