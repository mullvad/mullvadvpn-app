package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange

data class ShadowsocksSettingsState(
    val port: Constraint<Port> = Constraint.Any,
    val customPort: Port? = null,
    val validPortRanges: List<PortRange> = emptyList(),
)
