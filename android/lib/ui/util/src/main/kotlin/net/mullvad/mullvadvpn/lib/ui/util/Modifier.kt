package net.mullvad.mullvadvpn.lib.ui.util

import androidx.compose.ui.Modifier

fun <T> Modifier.applyIfNotNull(
    value: T?,
    and: Boolean = true,
    block: Modifier.(T) -> Modifier,
): Modifier =
    if (value != null && and) {
        this.then(Modifier.block(value))
    } else {
        this
    }

fun Modifier.applyIf(condition: Boolean, block: Modifier.() -> Modifier): Modifier =
    if (condition) {
        this.then(Modifier.block())
    } else {
        this
    }
