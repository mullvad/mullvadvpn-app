package net.mullvad.mullvadvpn.compose.state

import com.ramcosta.composedestinations.spec.BaseRoute
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType

data class SelectPortUiState(
    val portType: PortType,
    val port: Constraint<Port> = Constraint.Any,
    val customPort: Port? = null,
    val customPortEnabled: Boolean,
    val title: String,
    val allowedPortRanges: List<PortRange> = emptyList(),
    val presetPorts: List<Port> = emptyList(),
    val infoDestination: BaseRoute? = null,
) {
    val isCustom = port is Constraint.Only && port.value !in presetPorts
}
