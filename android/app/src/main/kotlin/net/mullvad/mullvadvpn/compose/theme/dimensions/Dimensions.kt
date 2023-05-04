package net.mullvad.mullvadvpn.compose.theme.dimensions

import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private val LocalDimen = compositionLocalOf { Dimensions() }
val Dimens: Dimensions
    @Composable
    get() = LocalDimen.current

data class Dimensions(
    val mediumPadding: Dp = 16.dp,
    val smallPadding: Dp = 8.dp,
    val listItemDivider: Dp = 1.dp,
    val listItemHeight: Dp = 50.dp,
    val loadingSpinnerSize: Dp = 24.dp,
    val loadingSpinnerStrokeWidth: Dp = 3.dp,
    val loadingSpinnerPadding: Dp = 12.dp
)
