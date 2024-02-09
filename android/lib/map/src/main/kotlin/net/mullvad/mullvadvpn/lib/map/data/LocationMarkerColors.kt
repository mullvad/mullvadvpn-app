package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.ui.graphics.Color

data class LocationMarkerColors(
    val centerColor: Color,
    val ringBorderColor: Color = Color.White,
    val shadowColor: Color = Color.Black.copy(alpha = 0.55f),
    val perimeterColors: Color = centerColor.copy(alpha = 0.4f)
)