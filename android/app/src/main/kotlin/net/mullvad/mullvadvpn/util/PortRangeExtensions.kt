package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.PortRange

fun List<PortRange>.portValid(port: Int) =
    this.any { portRange -> portRange.from <= port && portRange.to >= port }

fun List<PortRange>.asString() = buildString {
    this@asString.forEachIndexed { index, range ->
        if (index != 0) {
            append(",")
            append(" ")
        }
        if (range.from == range.to) {
            append(range.from)
        } else {
            append("${range.from}-${range.to}")
        }
    }
}
