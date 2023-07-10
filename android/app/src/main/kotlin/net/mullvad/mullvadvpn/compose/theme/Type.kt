package net.mullvad.mullvadvpn.compose.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/**
 * A collection of the fonts used in the app.
 *
 * The file contains all of the Material 3 typography classes, only some are used within the
 * MullvadVPN Android app.
 *
 * Typography used within the app:
 * - headlineLarge -> Used for titles, country and city in connection info. ex: VPN settings
 * - titleLarge -> Used for parent list-items. ex: DNS content blockers
 * - titleMedium -> Used for child and child-child list-items. ex: Automatic option in Tunnel
 *   protocol
 * - bodyMedium -> Used for body text. ex: Info text in dialogs
 * - labelLarge -> Used for button labels. ex: Got it!
 * - labelSmall -> Used for smaller descriptions under list-items and filters. ex: Filtered:
 */
internal val MullvadMaterial3Typography =
    Typography(
        displayLarge = TextStyle(fontFamily = FontFamily.SansSerif),
        displayMedium = TextStyle(fontFamily = FontFamily.SansSerif),
        displaySmall = TextStyle(fontFamily = FontFamily.SansSerif),
        headlineLarge =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.Bold,
                fontSize = 32.sp,
                lineHeight = 36.sp,
            ),
        headlineMedium = TextStyle(fontFamily = FontFamily.SansSerif),
        headlineSmall = TextStyle(fontFamily = FontFamily.SansSerif),
        titleLarge =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.Medium,
                fontSize = 20.sp,
                lineHeight = 28.sp,
            ),
        titleMedium =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.Normal,
                fontSize = 18.sp,
                lineHeight = 26.sp
            ),
        titleSmall = TextStyle(fontFamily = FontFamily.SansSerif),
        bodyLarge = TextStyle(fontFamily = FontFamily.SansSerif),
        bodyMedium =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.Normal,
                fontSize = 16.sp,
                lineHeight = 20.sp,
            ),
        bodySmall = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.SemiBold),
        labelLarge =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.SemiBold,
                fontSize = 18.sp,
                lineHeight = 23.sp
            ),
        labelMedium =
            TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.SemiBold),
        labelSmall =
            TextStyle(
                fontFamily = FontFamily.SansSerif,
                fontWeight = FontWeight.SemiBold,
                fontSize = 14.sp,
                lineHeight = 16.sp
            )
    )
