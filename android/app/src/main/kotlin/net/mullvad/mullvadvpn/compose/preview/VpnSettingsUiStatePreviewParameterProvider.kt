package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsUiState

private const val MTU = 1337
@Suppress("MagicNumber") private val PORT1 = Port(9001)
@Suppress("MagicNumber") private val PORT2 = Port(12433)

class VpnSettingsUiStatePreviewParameterProvider : PreviewParameterProvider<VpnSettingsUiState> {
    override val values =
        sequenceOf(
            VpnSettingsUiState.Loading(),
            VpnSettingsUiState.Content.from(
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
                selectedWireguardPort = Constraint.Any,
                customWireguardPort = PORT1,
                availablePortRanges = listOf(PORT1..PORT2),
                systemVpnSettingsAvailable = true,
                autoStartAndConnectOnBoot = true,
                isIpv6Enabled = true,
                obfuscationMode = ObfuscationMode.Udp2Tcp,
                selectedUdp2TcpObfuscationPort = Constraint.Any,
                selectedShadowsocksObfuscationPort = Constraint.Any,
                isContentBlockersExpanded = true,
                deviceIpVersion = Constraint.Any,
                isModal = false,
            ),
        )
}
