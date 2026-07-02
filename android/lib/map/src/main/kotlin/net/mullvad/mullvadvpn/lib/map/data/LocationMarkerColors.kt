package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.animation.core.EaseInCirc
import androidx.compose.animation.core.EaseOutQuad
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

@Immutable
data class LocationMarkerColors(
    val centerColor: Color,
    val ringBorderColor: Color = Color.White,
    val shadowColor: Color = Color.Black.copy(alpha = DEFAULT_SHADOW_ALPHA),
    val perimeterColors: Color? = centerColor.copy(alpha = DEFAULT_PERIMETER_ALPHA),
) {
    companion object {
        private const val DEFAULT_SHADOW_ALPHA = 0.55f
        private const val DEFAULT_PERIMETER_ALPHA = 0.4f

        fun default(alpha: Float = 1f) =
            LocationMarkerColors(
                perimeterColors = null,
                centerColor = Color(0xFF192E45.toInt()).copy(alpha = EaseOutQuad.transform(alpha)),
                ringBorderColor =
                    Color(0xFFFFFFFF.toInt()).copy(alpha = EaseInCirc.transform(alpha)),
                shadowColor = Color.Black.copy(DEFAULT_SHADOW_ALPHA * EaseInCirc.transform(alpha)),
            )

        fun hop(alpha: Float = 1f) =
            LocationMarkerColors(
                perimeterColors = Color.Transparent,
                centerColor = Color(0xFF44AD4D.toInt()).copy(alpha = EaseOutQuad.transform(alpha)),
                ringBorderColor =
                    Color(0xFFFFFFFF.toInt()).copy(alpha = EaseInCirc.transform(alpha)),
                shadowColor = Color.Black.copy(DEFAULT_SHADOW_ALPHA * EaseInCirc.transform(alpha)),
            )
    }
}
