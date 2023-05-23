package net.mullvad.mullvadvpn.compose.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.compose.theme.dimensions.Dimensions
import net.mullvad.mullvadvpn.compose.theme.dimensions.defaultDimensions
import net.mullvad.mullvadvpn.compose.theme.typeface.TypeScale

// Add our own definitions here
private val MullvadTypography =
    Typography(
        headlineSmall =
            TextStyle(
                color = MullvadWhite,
                fontSize = TypeScale.TextBig,
                fontWeight = FontWeight.Bold
            ),
        bodySmall = TextStyle(color = MullvadWhite, fontSize = TypeScale.TextSmall),
        titleMedium =
            TextStyle(fontSize = TypeScale.TextMediumPlus, fontWeight = FontWeight.SemiBold),
        labelLarge =
            TextStyle(
                fontWeight = FontWeight.Normal,
                letterSpacing = TextUnit.Unspecified,
                fontSize = TypeScale.TextHostname
            ),
        labelMedium =
            TextStyle(
                fontWeight =
                    FontWeight
                        .Normal, // TODO This should be semi-bold, but it's not possible to use at
                                 // the moment, since some other subtitles uses HmtlText which does
                                 // not support semi-bold font weight
                letterSpacing = TextUnit.Unspecified,
                fontSize = TypeScale.TextSmall
            )
    )

private val MullvadColorPalette =
    lightColorScheme(
        primary = MullvadBlue,
        secondary = MullvadDarkBlue,
        tertiary = MullvadRed,
        onSurfaceVariant = MullvadWhite,
        onPrimary = MullvadWhite
    )

val Shapes =
    Shapes(
        small = RoundedCornerShape(4.dp),
        medium = RoundedCornerShape(4.dp),
        large = RoundedCornerShape(0.dp)
    )

val Dimens: Dimensions
    @Composable get() = LocalAppDimens.current

@Composable
fun ProvideDimens(dimensions: Dimensions, content: @Composable () -> Unit) {
    val dimensionSet = remember { dimensions }
    CompositionLocalProvider(LocalAppDimens provides dimensionSet, content = content)
}

private val LocalAppDimens = staticCompositionLocalOf { defaultDimensions }

@Composable
fun AppTheme(content: @Composable () -> Unit) {
    val colors = MullvadColorPalette
    val typography = MullvadTypography
    // Set dimensions and type scale based on configurations here
    val dimensions = defaultDimensions

    ProvideDimens(dimensions = dimensions) {
        MaterialTheme(
            colorScheme = colors,
            shapes = Shapes,
            typography = typography,
            content = content
        )
    }
}
