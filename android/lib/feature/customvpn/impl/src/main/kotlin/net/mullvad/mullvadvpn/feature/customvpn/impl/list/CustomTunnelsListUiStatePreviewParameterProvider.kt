package net.mullvad.mullvadvpn.feature.customvpn.impl.list

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import java.net.InetSocketAddress
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.model.WireguardKey

class CustomTunnelsListUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, CustomTunnelsListUiState>> {
    override val values: Sequence<Lc<Unit, CustomTunnelsListUiState>> =
        sequenceOf(
            // Loading state
            Lc.Loading(Unit),
            // Default state
            Lc.Content(
                CustomTunnelsListUiState(
                    customVpnConfig = null,
                    tunnelStats = null,
                )
            ),
            Lc.Content(
                CustomTunnelsListUiState(
                    customVpnConfig =
                        CustomVpnConfig(
                            tunnelConfig =
                                TunnelConfig(
                                    privateKey =
                                        @Suppress("UnsafeCallOnNullableType")
                                        WireguardKey.from(
                                                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
                                            )
                                            .getOrNull()!!,
                                    tunnelIps = emptySet(),
                                ),
                            peerConfig =
                                listOf(
                                    PeerConfig(
                                        publicKey =
                                            @Suppress("UnsafeCallOnNullableType")
                                            WireguardKey.from(
                                                    "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE="
                                                )
                                                .getOrNull()!!,
                                        endpoint = InetSocketAddress(51230),
                                        allowedIps = emptySet(),
                                    )
                                ),
                            enabled = true,
                            name = "Test Tunnel",
                        ),
                    tunnelStats = TunnelStats(rx = 12000, tx = 5000),
                )
            ),
        )
}
