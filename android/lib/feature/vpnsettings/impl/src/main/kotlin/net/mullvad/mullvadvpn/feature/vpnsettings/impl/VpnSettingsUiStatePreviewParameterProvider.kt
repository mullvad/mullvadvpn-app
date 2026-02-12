package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
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
                    isCustomDnsEnabled = true,
                    customDnsItems =
                        listOf(CustomDnsItem(address = "0.0.0.0", isLocal = false, isIpv6 = false)),
                    contentBlockersOptions =
                        DefaultDnsOptions(
                            blockAds = true,
                            blockMalware = true,
                            blockGambling = true,
                            blockTrackers = true,
                            blockSocialMedia = true,
                            blockAdultContent = true,
                        ),
                    quantumResistant = QuantumResistantState.On,
                    systemVpnSettingsAvailable = true,
                    autoStartAndConnectOnBoot = true,
                    isIpv6Enabled = true,
                    obfuscationMode = ObfuscationMode.Udp2Tcp,
                    isContentBlockersExpanded = true,
                    deviceIpVersion = Constraint.Any,
                    isModal = false,
                )
                .toLc(),
        )
}
