package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port

data class ShadowsocksSettingsUiState(
    val port: Constraint<Port> = Constraint.Any,
    val customPort: Port? = null,
) {
    val isCustom = port is Constraint.Only && port.value == customPort
}
