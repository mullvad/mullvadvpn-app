package net.mullvad.mullvadvpn.compose.theme.typeface

import androidx.compose.ui.unit.sp

/**
 * Font sizes used by text styles in the app.
 *
 * NOTE:
 * * Do not use these font sizes directly. Instead use the styles defined in Typeface and/or the
 * standard styles in the material theme
 * * Order entries within each type by descending size.
 */
internal object TypeScale {
    val TextBig = 24.sp
    val TextMediumPlus = 18.sp
    val TextHostname = 15.sp
    val TextSmall = 13.sp
}
