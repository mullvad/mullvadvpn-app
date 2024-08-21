package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange

fun Constraint<Port>.hasValue(value: Int) =
    when (this) {
        is Constraint.Any -> false
        is Constraint.Only -> this.value.value == value
    }

fun Constraint<Port>.isCustom(presetPorts: List<Int>) =
    when (this) {
        is Constraint.Any -> false
        is Constraint.Only -> !presetPorts.contains(this.value.value)
    }

fun Constraint<Port>.toPortOrNull() =
    when (this) {
        is Constraint.Any -> null
        is Constraint.Only -> this.value
    }

fun Port.inAnyOf(portRanges: List<PortRange>): Boolean =
    portRanges.any { portRange -> this in portRange }

fun List<PortRange>.asString() = joinToString(", ", transform = PortRange::toFormattedString)
