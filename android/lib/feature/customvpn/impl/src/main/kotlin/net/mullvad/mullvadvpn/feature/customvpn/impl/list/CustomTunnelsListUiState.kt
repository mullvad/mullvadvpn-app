package net.mullvad.mullvadvpn.feature.customvpn.impl.list

import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats

data class CustomTunnelsListUiState(
    val customVpnConfig: CustomVpnConfig?,
    val tunnelStats: TunnelStats?,
)
