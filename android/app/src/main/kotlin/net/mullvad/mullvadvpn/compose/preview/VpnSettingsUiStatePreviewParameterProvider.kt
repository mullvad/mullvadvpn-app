package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.CustomDnsItem
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

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
                    customDnsItems = listOf(CustomDnsItem("0.0.0.0", false, false)),
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
                    isScrollToFeatureEnabled = true,
                )
                .toLc(),
        )
}
