package net.mullvad.mullvadvpn.compose.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Shapes
import androidx.compose.material.lightColors
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp


private val MullvadColorPalette = lightColors(
    primary = MullvadBlue,
    primaryVariant = MullvadDarkBlue,
    secondary = MullvadRed
)

val Shapes = Shapes(
    small = RoundedCornerShape(4.dp),
    medium = RoundedCornerShape(4.dp),
    large = RoundedCornerShape(0.dp)
)


@Composable
fun CollapsingToolbarTheme(
    content: @Composable () -> Unit
) {
    val colors = MullvadColorPalette

    MaterialTheme(
        colors = colors,
        shapes = Shapes,
        content = content
    )
}
