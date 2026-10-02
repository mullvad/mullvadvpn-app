package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

import net.mullvad.mullvadvpn.lib.model.PersonalVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats

data class CustomTunnelsListUiState(
    val customVpnConfigState: CustomTunnelConfigState?,
    val tunnelStats: TunnelStats?,
)

sealed interface CustomTunnelConfigState {
    val config: PersonalVpnConfig

    data class Enabled(override val config: PersonalVpnConfig) : CustomTunnelConfigState

    data class Disabled(override val config: PersonalVpnConfig) : CustomTunnelConfigState
}
