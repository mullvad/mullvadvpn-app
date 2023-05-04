package net.mullvad.mullvadvpn.compose.theme.dimensions

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

data class Dimensions(
    val mediumPadding: Dp = 16.dp,
    val smallPadding: Dp = 8.dp,
    val listItemDivider: Dp = 1.dp,
    val listItemHeight: Dp = 50.dp,
    val loadingSpinnerSize: Dp = 24.dp,
    val loadingSpinnerStrokeWidth: Dp = 3.dp,
    val loadingSpinnerPadding: Dp = 12.dp
)

val defaultDimensions = Dimensions()
// Add more configurations here if needed
