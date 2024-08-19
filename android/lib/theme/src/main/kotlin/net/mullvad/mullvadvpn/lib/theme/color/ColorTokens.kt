package net.mullvad.mullvadvpn.lib.theme.color

import androidx.compose.ui.graphics.Color

// This is experimental and currently not used
internal object ColorLightTokens {
    val Background = PaletteTokens.DarkBlue100
    val Error = PaletteTokens.Red600
    val ErrorContainer = PaletteTokens.Red100
    val InverseOnSurface = PaletteTokens.DarkBlue100
    val InversePrimary = PaletteTokens.Blue200
    val InverseSurface = PaletteTokens.DarkBlue800
    val OnBackground = PaletteTokens.DarkBlue900
    val OnError = PaletteTokens.Red900
    val OnErrorContainer = PaletteTokens.Red900
    val OnPrimary = PaletteTokens.Blue900
    val OnPrimaryContainer = PaletteTokens.Blue900
    val OnPrimaryFixed = PaletteTokens.Blue900
    val OnPrimaryFixedVariant = PaletteTokens.Blue700
    val OnSecondary = PaletteTokens.Green900
    val OnSecondaryContainer = PaletteTokens.Green900
    val OnSecondaryFixed = PaletteTokens.Green900
    val OnSecondaryFixedVariant = PaletteTokens.Green700
    val OnSurface = PaletteTokens.DarkBlue900
    val OnSurfaceVariant = PaletteTokens.DarkBlue700
    val OnTertiary = PaletteTokens.Yellow900
    val OnTertiaryContainer = PaletteTokens.Yellow900
    val OnTertiaryFixed = PaletteTokens.Yellow900
    val OnTertiaryFixedVariant = PaletteTokens.Yellow700
    val Outline = PaletteTokens.DarkBlue500
    val OutlineVariant = PaletteTokens.DarkBlue200
    val Primary = PaletteTokens.Blue600
    val PrimaryContainer = PaletteTokens.Blue100
    val PrimaryFixed = PaletteTokens.Blue100
    val PrimaryFixedDim = PaletteTokens.Blue200
    val Scrim = PaletteTokens.DarkBlue900
    val Secondary = PaletteTokens.Green600
    val SecondaryContainer = PaletteTokens.Green100
    val SecondaryFixed = PaletteTokens.Green100
    val SecondaryFixedDim = PaletteTokens.Green200
    val Surface = PaletteTokens.DarkBlue100
    val SurfaceBright = PaletteTokens.DarkBlue100
    val SurfaceContainer = PaletteTokens.DarkBlue900
    val SurfaceContainerHighest = OpacityTokens.WhiteOnBlue20
    val SurfaceContainerHigh = OpacityTokens.WhiteOnBlue40
    val SurfaceContainerLow = OpacityTokens.WhiteOnBlue50
    val SurfaceContainerLowest = OpacityTokens.WhiteOnBlue60
    val SurfaceDim = PaletteTokens.DarkBlue200
    val SurfaceTint = Primary
    val SurfaceVariant = PaletteTokens.DarkBlue100
    val Tertiary = PaletteTokens.Yellow600
    val TertiaryContainer = PaletteTokens.Yellow100
    val TertiaryFixed = PaletteTokens.Yellow100
    val TertiaryFixedDim = PaletteTokens.Yellow200
}

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
    val SurfaceDim = PaletteTokens.Black // Currently not used directly
    val SurfaceTint = Primary // Currently not used directly
    val SurfaceVariant = PaletteTokens.DarkBlue500 // Currently not used directly
    val Tertiary = PaletteTokens.Green500 // Connected and selected color
    val TertiaryContainer = OpacityTokens.WhiteOnDarkBlue10 // Used by text color
    val TertiaryFixed = PaletteTokens.Yellow600 // Not in material 3 library yet
    val TertiaryFixedDim = PaletteTokens.Yellow500 // Not in material 3 library yet
}
