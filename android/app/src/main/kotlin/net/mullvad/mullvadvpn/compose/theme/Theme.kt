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
import androidx.compose.ui.graphics.Color
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
        headlineLarge = TextStyle(fontSize = TypeScale.TextHuge, fontWeight = FontWeight.Bold),
        headlineSmall =
            TextStyle(
                color = MullvadWhite,
                fontSize = TypeScale.TextBig,
                fontWeight = FontWeight.Bold
            ),
        bodySmall = TextStyle(color = MullvadWhite, fontSize = TypeScale.TextSmall),
        titleSmall =
            TextStyle(
                color = MullvadWhite,
                fontSize = TypeScale.TextMedium,
                fontWeight = FontWeight.SemiBold
            ),
        titleMedium =
            TextStyle(
                color = MullvadWhite,
                fontSize = TypeScale.TextMediumPlus,
                fontWeight = FontWeight.SemiBold
            ),
        labelMedium =
            TextStyle(
                color = MullvadWhite60,
                fontSize = TypeScale.TextSmall,
                fontWeight = FontWeight.SemiBold
            ),
        labelLarge =
            TextStyle(
                fontWeight = FontWeight.Normal,
                letterSpacing = TextUnit.Unspecified,
                fontSize = TypeScale.TextMedium
            )
    )

private val MullvadColorPalette =
    lightColorScheme(
        primary = MullvadBlue,
        secondary = MullvadDarkBlue,
        tertiary = MullvadRed,
        background = MullvadDarkBlue,
        surface = MullvadGreen,
        primaryContainer = MullvadBlue40,
        secondaryContainer = MullvadBlue20,
        onBackground = MullvadWhite,
        onSurfaceVariant = MullvadWhite,
        onPrimary = MullvadWhite,
        onSecondary = MullvadWhite60,
        onError = MullvadWhite,
        onSurface = MullvadWhite,
        inversePrimary = MullvadGreen,
        error = MullvadRed,
        errorContainer = Color(0xFFFFD323),
        outlineVariant = Color.Transparent, // Used by divider
        inverseSurface = MullvadWhite
    )

val Shapes =
    Shapes(
        small = RoundedCornerShape(4.dp),
        medium = RoundedCornerShape(4.dp),
        large = RoundedCornerShape(0.dp),
        extraLarge = RoundedCornerShape(4.dp)
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
