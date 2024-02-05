package net.mullvad.mullvadvpn.lib.theme.color

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

internal val MullvadYellow = Color(0xFFFFD524)
internal val MullvadGreen = Color(0xFF44AD4D)
internal val MullvadWhite60 = Color(0x99FFFFFF)
internal val MullvadWhite = Color(0xFFFFFFFF)
internal val MullvadRed = Color(0xFFE34039)
internal val MullvadDarkBlue = Color(0xFF192E45)

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
