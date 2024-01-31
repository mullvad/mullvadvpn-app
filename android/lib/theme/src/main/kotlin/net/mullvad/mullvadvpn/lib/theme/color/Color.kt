package net.mullvad.mullvadvpn.lib.theme.color

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

internal val MullvadYellow = Color(0xFFFFD524)

@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadBlue = Color(0xFF294D73)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadDarkBlue = Color(0xFF192E45)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadGreen = Color(0xFF44AD4D)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadRed = Color(0xFFE34039)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadWhite = Color(0xFFFFFFFF)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadWhite10 = Color(0x1AFFFFFF)
@Deprecated(
    "Deprecated for external usage and will be marked as internal in the future. Use material colors instead."
)
val MullvadWhite60 = Color(0x99FFFFFF)

const val AlphaVisible = 1f
const val AlphaDisabled = 0.2f
const val Alpha20 = 0.2f
const val AlphaInactive = 0.4f
const val Alpha40 = 0.4f
const val AlphaDescription = 0.6f
const val AlphaDisconnectButton = 0.6f
const val AlphaChevron = 0.6f
const val AlphaScrollbar = 0.6f
const val AlphaTopBar = 0.8f
const val AlphaInvisible = 0f

// Custom colors, they only link to normal material 3 colors for now
val ColorScheme.variant: Color
    @Composable get() = MaterialTheme.colorScheme.surface
val ColorScheme.onVariant: Color
    @Composable get() = MaterialTheme.colorScheme.onSurface

val ColorScheme.selected: Color
    @Composable get() = MaterialTheme.colorScheme.surface
