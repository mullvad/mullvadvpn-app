package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsUiState

private const val MTU = 1337
@Suppress("MagicNumber") private val PORT1 = Port(9001)
@Suppress("MagicNumber") private val PORT2 = Port(12433)

class VpnSettingsUiStatePreviewParameterProvider : PreviewParameterProvider<VpnSettingsUiState> {
    override val values =
        sequenceOf(
            VpnSettingsUiState.Loading(),
            VpnSettingsUiState.Content(
                emptyList()
                //                mtu = Mtu(MTU),
                //                isLocalNetworkSharingEnabled = true,
                //                isCustomDnsEnabled = true,
                //                customDnsItems = listOf(CustomDnsItem("0.0.0.0", false, false)),
                //                contentBlockersOptions =
                //                    DefaultDnsOptions(
                //                        blockAds = true,
                //                        blockMalware = true,
                //                        blockGambling = true,
                //                        blockTrackers = true,
                //                        blockSocialMedia = true,
                //                        blockAdultContent = true,
                //                    ),
                //                quantumResistant = QuantumResistantState.On,
                //                selectedWireguardPort = Constraint.Any,
                //                customWireguardPort = PORT1,
                //                availablePortRanges = listOf(PORT1..PORT2),
                //                systemVpnSettingsAvailable = true,
                //                autoStartAndConnectOnBoot = true,
            ),
        )
}
