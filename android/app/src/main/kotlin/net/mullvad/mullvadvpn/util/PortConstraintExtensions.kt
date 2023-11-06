package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port

fun Constraint<Port>.hasValue(value: Int) =
    when (this) {
        is Constraint.Any -> false
        is Constraint.Only -> this.value.value == value
    }

fun Constraint<Port>.isCustom() =
    when (this) {
        is Constraint.Any -> false
        is Constraint.Only -> !WIREGUARD_PRESET_PORTS.contains(this.value.value)
    }

fun Constraint<Port>.toValueOrNull() =
    when (this) {
        is Constraint.Any -> null
        is Constraint.Only -> this.value.value
    }
