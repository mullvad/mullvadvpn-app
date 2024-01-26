package net.mullvad.mullvadvpn.lib.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.color.MullvadBlue20
import net.mullvad.mullvadvpn.lib.theme.color.MullvadDarkBlue
import net.mullvad.mullvadvpn.lib.theme.color.MullvadGreen
import net.mullvad.mullvadvpn.lib.theme.color.MullvadRed
import net.mullvad.mullvadvpn.lib.theme.color.MullvadWhite
import net.mullvad.mullvadvpn.lib.theme.color.MullvadWhite60
import net.mullvad.mullvadvpn.lib.theme.color.MullvadYellow
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_error
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onError
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onPrimary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onSurfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onTertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_primary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_surfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_tertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.dimensions.Dimensions
import net.mullvad.mullvadvpn.lib.theme.dimensions.defaultDimensions
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale

// Add our own definitions here
private val MullvadTypography =
    Typography(
        headlineLarge = TextStyle(fontSize = TypeScale.TextHuge, fontWeight = FontWeight.Bold),
        headlineSmall = TextStyle(fontSize = TypeScale.TextBig, fontWeight = FontWeight.Bold),
        bodySmall = TextStyle(fontSize = TypeScale.TextSmall),
        titleSmall = TextStyle(fontSize = TypeScale.TextMedium, fontWeight = FontWeight.SemiBold),
        bodyMedium = TextStyle(fontSize = TypeScale.TextMediumPlus, fontWeight = FontWeight.Bold),
        titleMedium =
            TextStyle(fontSize = TypeScale.TextMediumPlus, fontWeight = FontWeight.SemiBold),
        titleLarge = TextStyle(fontSize = TypeScale.TitleLarge, fontFamily = FontFamily.SansSerif),
        labelMedium = TextStyle(fontSize = TypeScale.TextSmall, fontWeight = FontWeight.SemiBold),
        labelLarge =
            TextStyle(
                fontWeight = FontWeight.Normal,
                letterSpacing = TextUnit.Unspecified,
                fontSize = TypeScale.TextMedium
            )
    )

private val darkColorScheme =
    darkColorScheme(
        primary = md_theme_dark_primary,
        onPrimary = md_theme_dark_onPrimary,
        // primaryContainer = md_theme_dark_primaryContainer,
        // onPrimaryContainer = md_theme_dark_onPrimaryContainer,
        secondary = MullvadDarkBlue,
        onSecondary = MullvadWhite60,
        secondaryContainer = MullvadBlue20,
        // onSecondaryContainer = md_theme_dark_onSecondaryContainer,
        tertiary = MullvadRed,
        // onTertiary = md_theme_dark_onTertiary,
        tertiaryContainer = md_theme_dark_tertiaryContainer,
        onTertiaryContainer = md_theme_dark_onTertiaryContainer,
        error = md_theme_dark_error,
        errorContainer = MullvadYellow,
        onError = md_theme_dark_onError,
        // onErrorContainer = md_theme_dark_onErrorContainer,
        background = MullvadDarkBlue,
        onBackground = MullvadWhite,
        surface = MullvadGreen,
        onSurface = MullvadWhite,
        surfaceVariant = md_theme_dark_surfaceVariant,
        onSurfaceVariant = md_theme_dark_onSurfaceVariant,
        // outline = md_theme_dark_outline,
        // inverseOnSurface = md_theme_dark_inverseOnSurface,
        inverseSurface = MullvadWhite,
        inversePrimary = MullvadGreen,
        // surfaceTint = md_theme_dark_surfaceTint,
        outlineVariant = Color.Transparent, // Used by divider,
        // scrim = md_theme_dark_scrim,
    )

val Shapes =
    Shapes(
        small = RoundedCornerShape(4.dp),
        medium = RoundedCornerShape(4.dp),
        large = RoundedCornerShape(0.dp),
        extraLarge = RoundedCornerShape(11.dp)
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
    val colors = darkColorScheme
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
