package net.mullvad.mullvadvpn.compose.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp

private val MullvadColorPalette =
    lightColorScheme(primary = MullvadBlue, secondary = MullvadDarkBlue, tertiary = MullvadRed, onSurfaceVariant = MullvadWhite)

val Shapes =
    Shapes(
        small = RoundedCornerShape(4.dp),
        medium = RoundedCornerShape(4.dp),
        large = RoundedCornerShape(0.dp)
    )

@Composable
fun AppTheme(content: @Composable () -> Unit) {
    val colors = MullvadColorPalette

    MaterialTheme(colorScheme = colors, shapes = Shapes, content = content)
}
