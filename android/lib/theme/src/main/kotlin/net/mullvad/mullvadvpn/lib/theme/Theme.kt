package net.mullvad.mullvadvpn.lib.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.ripple.RippleAlpha
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LocalRippleConfiguration
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RippleConfiguration
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.lib.theme.color.ColorDarkTokens
import net.mullvad.mullvadvpn.lib.theme.color.ColorLightTokens
import net.mullvad.mullvadvpn.lib.theme.dimensions.Dimensions
import net.mullvad.mullvadvpn.lib.theme.dimensions.defaultDimensions
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale

// Add our own definitions here
private val MullvadTypography =
    Typography(
        headlineLarge = TextStyle(fontSize = TypeScale.TextHuge, fontWeight = FontWeight.Bold),
        headlineMedium =
            TextStyle(
                fontWeight = FontWeight.Bold,
                fontSize = TypeScale.TextHeadline,
                lineHeight = TypeScale.HeadlineMediumLineHeight,
            ),
        headlineSmall = TextStyle(fontSize = TypeScale.TextBig, fontWeight = FontWeight.Bold),
        bodySmall = TextStyle(fontSize = TypeScale.TextSmall, lineHeight = TypeScale.TextMediumPlus),
        titleSmall = TextStyle(fontSize = TypeScale.TextMedium, fontWeight = FontWeight.SemiBold),
        titleMedium =
            TextStyle(fontSize = TypeScale.TextMediumPlus, fontWeight = FontWeight.SemiBold),
        titleLarge = TextStyle(fontSize = TypeScale.TitleLarge, fontFamily = FontFamily.SansSerif),
        labelMedium = TextStyle(fontSize = TypeScale.TextSmall, fontWeight = FontWeight.SemiBold),
        labelLarge =
            TextStyle(
                fontWeight = FontWeight.Normal,
                letterSpacing = TextUnit.Unspecified,
                fontSize = TypeScale.TextMedium,
            ),
    )

private val lightColorScheme =
    lightColorScheme(
        primary = ColorLightTokens.Primary,
        onPrimary = ColorLightTokens.OnPrimary,
        primaryContainer = ColorLightTokens.PrimaryContainer,
        onPrimaryContainer = ColorLightTokens.OnPrimaryContainer,
        inversePrimary = ColorLightTokens.InversePrimary,
        secondary = ColorLightTokens.Secondary,
        onSecondary = ColorLightTokens.OnSecondary,
        secondaryContainer = ColorLightTokens.SecondaryContainer,
        onSecondaryContainer = ColorLightTokens.OnSecondaryContainer,
        tertiary = ColorLightTokens.Tertiary,
        onTertiary = ColorLightTokens.OnTertiary,
        tertiaryContainer = ColorLightTokens.TertiaryContainer,
        onTertiaryContainer = ColorLightTokens.OnTertiaryContainer,
        background = ColorLightTokens.Background,
        onBackground = ColorLightTokens.OnBackground,
        surface = ColorLightTokens.Surface,
        onSurface = ColorLightTokens.OnSurface,
        surfaceVariant = ColorLightTokens.SurfaceVariant,
        onSurfaceVariant = ColorLightTokens.OnSurfaceVariant,
        surfaceTint = ColorLightTokens.SurfaceTint,
        inverseSurface = ColorLightTokens.InverseSurface,
        inverseOnSurface = ColorLightTokens.InverseOnSurface,
        error = ColorLightTokens.Error,
        onError = ColorLightTokens.OnError,
        errorContainer = ColorLightTokens.ErrorContainer,
        onErrorContainer = ColorLightTokens.OnErrorContainer,
        outline = ColorLightTokens.Outline,
        outlineVariant = ColorLightTokens.OutlineVariant,
        scrim = ColorLightTokens.Scrim,
        surfaceBright = ColorLightTokens.SurfaceBright,
        surfaceContainer = ColorLightTokens.SurfaceContainer,
        surfaceContainerHigh = ColorLightTokens.SurfaceContainerHigh,
        surfaceContainerHighest = ColorLightTokens.SurfaceContainerHighest,
        surfaceContainerLow = ColorLightTokens.SurfaceContainerLow,
        surfaceContainerLowest = ColorLightTokens.SurfaceContainerLowest,
        surfaceDim = ColorLightTokens.SurfaceDim,
    )

private val darkColorScheme =
    darkColorScheme(
        primary = ColorDarkTokens.Primary,
        onPrimary = ColorDarkTokens.OnPrimary,
        primaryContainer = ColorDarkTokens.PrimaryContainer,
        onPrimaryContainer = ColorDarkTokens.OnPrimaryContainer,
        inversePrimary = ColorDarkTokens.InversePrimary,
        secondary = ColorDarkTokens.Secondary,
        onSecondary = ColorDarkTokens.OnSecondary,
        secondaryContainer = ColorDarkTokens.SecondaryContainer,
        onSecondaryContainer = ColorDarkTokens.OnSecondaryContainer,
        tertiary = ColorDarkTokens.Tertiary,
        onTertiary = ColorDarkTokens.OnTertiary,
        tertiaryContainer = ColorDarkTokens.TertiaryContainer,
        onTertiaryContainer = ColorDarkTokens.OnTertiaryContainer,
        background = ColorDarkTokens.Background,
        onBackground = ColorDarkTokens.OnBackground,
        surface = ColorDarkTokens.Surface,
        onSurface = ColorDarkTokens.OnSurface,
        surfaceVariant = ColorDarkTokens.SurfaceVariant,
        onSurfaceVariant = ColorDarkTokens.OnSurfaceVariant,
        surfaceTint = ColorDarkTokens.SurfaceTint,
        inverseSurface = ColorDarkTokens.InverseSurface,
        inverseOnSurface = ColorDarkTokens.InverseOnSurface,
        error = ColorDarkTokens.Error,
        onError = ColorDarkTokens.OnError,
        errorContainer = ColorDarkTokens.ErrorContainer,
        onErrorContainer = ColorDarkTokens.OnErrorContainer,
        outline = ColorDarkTokens.Outline,
        outlineVariant = ColorDarkTokens.OutlineVariant,
        scrim = ColorDarkTokens.Scrim,
        surfaceBright = ColorDarkTokens.SurfaceBright,
        surfaceContainer = ColorDarkTokens.SurfaceContainer,
        surfaceContainerHigh = ColorDarkTokens.SurfaceContainerHigh,
        surfaceContainerHighest = ColorDarkTokens.SurfaceContainerHighest,
        surfaceContainerLow = ColorDarkTokens.SurfaceContainerLow,
        surfaceContainerLowest = ColorDarkTokens.SurfaceContainerLowest,
        surfaceDim = ColorDarkTokens.SurfaceDim,
    )

val Shapes =
    Shapes(
        small = RoundedCornerShape(4.dp),
        medium = RoundedCornerShape(4.dp),
        large = RoundedCornerShape(12.dp),
        extraLarge = RoundedCornerShape(11.dp),
    )

val Dimens: Dimensions
    @Composable get() = LocalAppDimens.current

private object StateTokens {
    const val DraggedStateLayerOpacity = 0.16f // 0.16f (Material default)
    const val FocusStateLayerOpacity = 0.24f // 0.12f (Material default)
    const val HoverStateLayerOpacity = 0.08f // 0.08f (Material default)
    const val PressedStateLayerOpacity = 0.12f // 0.12f (Material default)
}

private val rippleAlpha =
    RippleAlpha(
        pressedAlpha = StateTokens.PressedStateLayerOpacity,
        focusedAlpha = StateTokens.FocusStateLayerOpacity,
        draggedAlpha = StateTokens.DraggedStateLayerOpacity,
        hoveredAlpha = StateTokens.HoverStateLayerOpacity,
    )

@Composable
fun ProvideDimens(dimensions: Dimensions, content: @Composable () -> Unit) {
    val dimensionSet = remember { dimensions }
    CompositionLocalProvider(LocalAppDimens provides dimensionSet, content = content)
}

private val LocalAppDimens = staticCompositionLocalOf { defaultDimensions }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AppTheme(content: @Composable () -> Unit) {
    val typography = MullvadTypography
    // Set dimensions and type scale based on configurations here
    val dimensions = defaultDimensions

    ProvideDimens(dimensions = dimensions) {
        MaterialTheme(
            colorScheme = darkColorScheme,
            shapes = Shapes,
            typography = typography,
            content = {
                CompositionLocalProvider(
                    LocalRippleConfiguration provides RippleConfiguration(rippleAlpha = rippleAlpha)
                ) {
                    content()
                }
            },
        )
    }
}
