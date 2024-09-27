package net.mullvad.mullvadvpn.lib.theme.color

import androidx.compose.ui.graphics.Color

internal object ColorDarkTokens {
    val Background = PaletteTokens.DarkBlue500 // Used by login screen and text fields
    val Error = PaletteTokens.Red500 // General error color
    val ErrorContainer = PaletteTokens.Red500 // Currently not used directly
    val InverseOnSurface = PaletteTokens.White // Currently not used directly
    val InversePrimary = PaletteTokens.Green500 // Currently not used directly (old selected color)
    val InverseSurface = PaletteTokens.White // Used by text fields
    val OnBackground = PaletteTokens.White // Used by some text on the login screen
    val OnError = PaletteTokens.White // Text that is displayed on error
    val OnErrorContainer = PaletteTokens.White // Currently not used directly
    val OnPrimary = PaletteTokens.White // Text that is displayed on primary (f.e. toolbar)
    val OnPrimaryContainer = PaletteTokens.White // Currently not used directly
    val OnPrimaryFixed = PaletteTokens.Blue50 // Not in material 3 library yet
    val OnPrimaryFixedVariant = PaletteTokens.Blue50 // Not in material 3 library yet
    val OnSecondary = PaletteTokens.White // Used by text fields
    val OnSecondaryContainer = OpacityTokens.WhiteOnDarkBlue60 // Currently not used directly
    val OnSecondaryFixed = PaletteTokens.Green50 // Not in material 3 library yet
    val OnSecondaryFixedVariant = PaletteTokens.Green50 // Not in material 3 library yet
    val OnSurface = PaletteTokens.White // Text that is displayed on the background
    val OnSurfaceVariant = OpacityTokens.WhiteOnDarkBlue60 // Description texts
    val OnTertiary = PaletteTokens.White // Text that is displayed on selected and connect
    val OnTertiaryContainer =
        Color(0xffacb4bc) // Used by text fields, will be replaced in the future
    val OnTertiaryFixed = PaletteTokens.Yellow50 // Not in material 3 library yet
    val OnTertiaryFixedVariant = PaletteTokens.Yellow50 // Not in material 3 library yet
    val Outline = PaletteTokens.Black // Currently not used directly
    val OutlineVariant = PaletteTokens.DarkBlue500 // Currently not used directly
    val Primary = PaletteTokens.Blue500 // Toolbar and top level cells
    val PrimaryContainer = PaletteTokens.Black // Currently not used directly
    val PrimaryFixed = PaletteTokens.Blue100 // Not in material 3 library yet
    val PrimaryFixedDim = PaletteTokens.Blue100 // Not in material 3 library yet
    val Scrim = PaletteTokens.Black // Overlay used by bottom sheet
    val Secondary = PaletteTokens.AlertBlue500 // Currently not used directly
    val SecondaryContainer = PaletteTokens.AlertBlue500 // Currently not used directly
    val SecondaryFixed = PaletteTokens.Green100 // Not in material 3 library yet
    val SecondaryFixedDim = PaletteTokens.Green100 // Not in material 3 library yet
    val Surface = PaletteTokens.DarkBlue500 // Background
    val SurfaceBright = PaletteTokens.DarkBlue700 // Currently not used directly
    val SurfaceContainer =
        PaletteTokens.AlertBlue500 // Background for in-app notification, bottom sheet
    val SurfaceContainerHighest = OpacityTokens.BlueOnDarkBlue60 // Second level cell / Sub cell
    val SurfaceContainerHigh = OpacityTokens.BlueOnDarkBlue40 // Third level cell
    val SurfaceContainerLow = OpacityTokens.BlueOnDarkBlue20 // Fourth level cell
    val SurfaceContainerLowest = OpacityTokens.BlueOnDarkBlue10 // Fifth level cell
    val SurfaceDim = PaletteTokens.Dim // Used only by cards in appearance screen
    val SurfaceTint = Primary // Currently not used directly
    val SurfaceVariant = PaletteTokens.DarkBlue500 // Currently not used directly
    val Tertiary = PaletteTokens.Green500 // Connected and selected color
    val TertiaryContainer = OpacityTokens.WhiteOnDarkBlue10 // Used by text color
    val TertiaryFixed = PaletteTokens.Yellow600 // Not in material 3 library yet
    val TertiaryFixedDim = PaletteTokens.Yellow500 // Not in material 3 library yet
}
