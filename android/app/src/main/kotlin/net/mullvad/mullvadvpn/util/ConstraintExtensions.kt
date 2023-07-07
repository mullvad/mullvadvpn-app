package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port

fun Constraint<Port>.hasValue(value: Int) =
    when (this) {
        is Constraint.Any -> false
        is Constraint.Only -> this.value.value == value
    }
