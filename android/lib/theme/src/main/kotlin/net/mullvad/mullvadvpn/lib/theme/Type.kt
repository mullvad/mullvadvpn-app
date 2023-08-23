package net.mullvad.mullvadvpn.lib.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

internal val MullvadMaterial3Typography =
    Typography(
        displayLarge = TextStyle(fontFamily = FontFamily.SansSerif),
        displayMedium = TextStyle(fontFamily = FontFamily.SansSerif),
        displaySmall = TextStyle(fontFamily = FontFamily.SansSerif),
        headlineLarge = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.Black),
        headlineMedium = TextStyle(fontFamily = FontFamily.SansSerif),
        headlineSmall = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.Black),
        titleLarge = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.Black),
        titleMedium = TextStyle(fontFamily = FontFamily.SansSerif),
        titleSmall = TextStyle(fontFamily = FontFamily.SansSerif),
        bodyLarge = TextStyle(fontFamily = FontFamily.SansSerif),
        bodyMedium = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.SemiBold),
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
        labelSmall = TextStyle(fontFamily = FontFamily.SansSerif, fontWeight = FontWeight.SemiBold)
    )
