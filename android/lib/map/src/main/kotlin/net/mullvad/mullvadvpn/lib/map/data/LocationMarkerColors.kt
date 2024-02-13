package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

@Immutable
data class LocationMarkerColors(
    val centerColor: Color,
    val ringBorderColor: Color = Color.White,
    val shadowColor: Color = Color.Black.copy(alpha = DEFAULT_SHADOW_ALPHA),
    val perimeterColors: Color = centerColor.copy(alpha = DEFAULT_PERIMETER_ALPHA)
) {
    companion object {
        const val DEFAULT_SHADOW_ALPHA = 0.55f
        const val DEFAULT_PERIMETER_ALPHA = 0.4f
    }
}
