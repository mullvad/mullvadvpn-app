package net.mullvad.mullvadvpn.lib.ui.util

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
fun <T> Modifier.applyIfNotNull(
    value: T?,
    and: Boolean = true,
    block: @Composable Modifier.(T) -> Modifier,
): Modifier =
    if (value != null && and) {
        this.then(Modifier.block(value))
    } else {
        this
    }

@Composable
fun Modifier.applyIf(condition: Boolean, block: @Composable Modifier.() -> Modifier): Modifier =
    if (condition) {
        this.then(Modifier.block())
    } else {
        this
    }
