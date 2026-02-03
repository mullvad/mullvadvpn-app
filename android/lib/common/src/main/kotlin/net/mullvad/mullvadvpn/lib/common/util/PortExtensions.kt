package net.mullvad.mullvadvpn.lib.common.util

import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange

fun Port.inAnyOf(portRanges: List<PortRange>): Boolean =
    portRanges.any { portRange -> this in portRange }

fun List<PortRange>.asString() = joinToString(", ", transform = PortRange::toFormattedString)
