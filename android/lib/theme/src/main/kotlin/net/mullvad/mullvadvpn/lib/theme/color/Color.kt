package net.mullvad.mullvadvpn.lib.theme.color

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.MenuItemColors
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

const val AlphaVisible = 1f
const val Alpha10 = 0.1f
const val AlphaDisabled = 0.2f
const val Alpha20 = 0.2f
const val AlphaInactive = 0.4f
const val Alpha40 = 0.4f
const val AlphaDescription = 0.6f
const val AlphaDisconnectButton = 0.6f
const val AlphaChevron = 0.6f
const val AlphaScrollbar = 0.6f
const val Alpha60 = 0.6f
const val AlphaTopBar = 0.8f
const val AlphaInvisible = 0f

// Custom colors, they only link to normal material 3 colors for now
val ColorScheme.variant: Color
    @Composable get() = MaterialTheme.colorScheme.secondary
val ColorScheme.onVariant: Color
    @Composable get() = MaterialTheme.colorScheme.onSecondary

val ColorScheme.selected: Color
    @Composable get() = MaterialTheme.colorScheme.secondaryContainer

val ColorScheme.onSelected: Color
    @Composable get() = MaterialTheme.colorScheme.onSecondaryContainer

val menuItemColors: MenuItemColors
    @Composable
    get() =
        MenuDefaults.itemColors()
            .copy(
                leadingIconColor = MaterialTheme.colorScheme.onSurface,
                textColor = MaterialTheme.colorScheme.onSurface,
            )
