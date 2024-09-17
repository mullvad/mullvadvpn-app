package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port

data class Udp2TcpSettingsState(val port: Constraint<Port> = Constraint.Any)
