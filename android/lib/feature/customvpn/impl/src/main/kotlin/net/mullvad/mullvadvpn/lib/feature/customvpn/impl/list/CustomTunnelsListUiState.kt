package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats

data class CustomTunnelsListUiState(
    val customVpnConfigState: CustomTunnelConfigState?,
    val tunnelStats: TunnelStats?,
)

sealed interface CustomTunnelConfigState {
    val config: CustomVpnConfig

    data class Enabled(override val config: CustomVpnConfig) : CustomTunnelConfigState

    data class Disabled(override val config: CustomVpnConfig) : CustomTunnelConfigState
}
