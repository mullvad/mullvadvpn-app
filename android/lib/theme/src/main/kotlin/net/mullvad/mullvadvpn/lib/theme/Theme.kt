package net.mullvad.mullvadvpn.lib.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.ripple.LocalRippleTheme
import androidx.compose.material.ripple.RippleAlpha
import androidx.compose.material.ripple.RippleTheme
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_background
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_error
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_errorContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_inverseOnSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_inversePrimary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_inverseSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onBackground
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onError
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onErrorContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onPrimary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onPrimaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onSecondary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onSecondaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onSurfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onTertiary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_onTertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_outline
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_outlineVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_primary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_primaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_scrim
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_secondary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_secondaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_surface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_surfaceTint
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_surfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_tertiary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_dark_tertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_background
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_error
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_errorContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_inverseOnSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_inversePrimary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_inverseSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onBackground
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onError
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onErrorContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onPrimary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onPrimaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onSecondary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onSecondaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onSurface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onSurfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onTertiary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_onTertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_outline
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_outlineVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_primary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_primaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_scrim
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_secondary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_secondaryContainer
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_surface
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_surfaceTint
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_surfaceVariant
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_tertiary
import net.mullvad.mullvadvpn.lib.theme.color.md_theme_light_tertiaryContainer
import net.mullvad.mullvadvpn.lib.theme.dimensions.Dimensions
import net.mullvad.mullvadvpn.lib.theme.dimensions.defaultDimensions
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale
import org.koin.core.context.GlobalContext.get

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

private val lightColorScheme =
    lightColorScheme(
        primary = md_theme_light_primary,
        onPrimary = md_theme_light_onPrimary,
        primaryContainer = md_theme_light_primaryContainer,
        onPrimaryContainer = md_theme_light_onPrimaryContainer,
        secondary = md_theme_light_secondary,
        onSecondary = md_theme_light_onSecondary,
        secondaryContainer = md_theme_light_secondaryContainer,
        onSecondaryContainer = md_theme_light_onSecondaryContainer,
        tertiary = md_theme_light_tertiary,
        onTertiary = md_theme_light_onTertiary,
        tertiaryContainer = md_theme_light_tertiaryContainer,
        onTertiaryContainer = md_theme_light_onTertiaryContainer,
        error = md_theme_light_error,
        errorContainer = md_theme_light_errorContainer,
        onError = md_theme_light_onError,
        onErrorContainer = md_theme_light_onErrorContainer,
        background = md_theme_light_background,
        onBackground = md_theme_light_onBackground,
        surface = md_theme_light_surface,
        onSurface = md_theme_light_onSurface,
        surfaceVariant = md_theme_light_surfaceVariant,
        onSurfaceVariant = md_theme_light_onSurfaceVariant,
        outline = md_theme_light_outline,
        inverseOnSurface = md_theme_light_inverseOnSurface,
        inverseSurface = md_theme_light_inverseSurface,
        inversePrimary = md_theme_light_inversePrimary,
        surfaceTint = md_theme_light_surfaceTint,
        outlineVariant = md_theme_light_outlineVariant,
        scrim = md_theme_light_scrim,
    )

private val darkColorScheme =
    darkColorScheme(
        primary = md_theme_dark_primary,
        onPrimary = md_theme_dark_onPrimary,
        primaryContainer = md_theme_dark_primaryContainer,
        onPrimaryContainer = md_theme_dark_onPrimaryContainer,
        secondary = md_theme_dark_secondary,
        onSecondary = md_theme_dark_onSecondary,
        secondaryContainer = md_theme_dark_secondaryContainer,
        onSecondaryContainer = md_theme_dark_onSecondaryContainer,
        tertiary = md_theme_dark_tertiary,
        onTertiary = md_theme_dark_onTertiary,
        tertiaryContainer = md_theme_dark_tertiaryContainer,
        onTertiaryContainer = md_theme_dark_onTertiaryContainer,
        error = md_theme_dark_error,
        errorContainer = md_theme_dark_errorContainer,
        onError = md_theme_dark_onError,
        onErrorContainer = md_theme_dark_onErrorContainer,
        background = md_theme_dark_background,
        onBackground = md_theme_dark_onBackground,
        surface = md_theme_dark_surface,
        onSurface = md_theme_dark_onSurface,
        surfaceVariant = md_theme_dark_surfaceVariant,
        onSurfaceVariant = md_theme_dark_onSurfaceVariant,
        outline = md_theme_dark_outline,
        inverseOnSurface = md_theme_dark_inverseOnSurface,
        inverseSurface = md_theme_dark_inverseSurface,
        inversePrimary = md_theme_dark_inversePrimary,
        surfaceTint = md_theme_dark_surfaceTint,
        outlineVariant = md_theme_dark_outlineVariant,
        scrim = md_theme_dark_scrim,
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
        hoveredAlpha = StateTokens.HoverStateLayerOpacity
    )

@Composable
fun ProvideDimens(dimensions: Dimensions, content: @Composable () -> Unit) {
    val dimensionSet = remember { dimensions }
    CompositionLocalProvider(LocalAppDimens provides dimensionSet, content = content)
}

private val LocalAppDimens = staticCompositionLocalOf { defaultDimensions }

@Composable
fun AppTheme(content: @Composable () -> Unit) {
    val themeRepository: ThemeRepository = get().get()
    val dynamicColor = themeRepository.useMaterialYouTheme().collectAsState().value
    val darkTheme =
        when (themeRepository.useDarkTheme().collectAsState().value) {
            DarkThemeState.SYSTEM -> isSystemInDarkTheme()
            DarkThemeState.ON -> true
            DarkThemeState.OFF -> false
        }
    val context = LocalContext.current
    val colorScheme =
        when {
            dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
                if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
            }
            darkTheme -> darkColorScheme
            else -> lightColorScheme
        }

    val typography = MullvadTypography
    // Set dimensions and type scale based on configurations here
    val dimensions = defaultDimensions

    ProvideDimens(dimensions = dimensions) {
        MaterialTheme(
            colorScheme = colorScheme.switch(),
            shapes = Shapes,
            typography = typography,
            content = {
                CompositionLocalProvider(LocalRippleTheme provides MullvadRippleTheme) { content() }
            }
        )
    }
}

@Immutable
object MullvadRippleTheme : RippleTheme {
    @Composable override fun defaultColor() = LocalContentColor.current

    @Composable override fun rippleAlpha() = rippleAlpha
}
