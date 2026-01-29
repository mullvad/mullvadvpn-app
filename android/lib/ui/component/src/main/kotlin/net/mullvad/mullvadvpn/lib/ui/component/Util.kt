package net.mullvad.mullvadvpn.lib.ui.component

import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

fun Collection<Any>.positionForIndex(index: Int): Position =
    when {
        size <= 1 -> Position.Single
        index == 0 -> Position.Top
        index == size - 1 -> Position.Bottom
        else -> Position.Middle
    }
