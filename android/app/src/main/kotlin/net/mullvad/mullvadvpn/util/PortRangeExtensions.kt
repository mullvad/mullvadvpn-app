package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange

fun List<PortRange>.isPortAnyOfRanges(port: Port): Boolean = any { portRange -> port in portRange }

fun List<PortRange>.asString() = joinToString(", ", transform = PortRange::toFormattedString)
