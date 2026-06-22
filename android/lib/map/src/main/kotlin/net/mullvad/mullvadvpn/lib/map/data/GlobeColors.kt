package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.map.internal.toFloatArray

@Immutable
data class GlobeColors(
    val landColor: Color,
    val oceanColor: Color,
    val contourColor: Color = oceanColor,
    val backgroundColor: Color = oceanColor,
) {
    val landColorArray = landColor.toFloatArray()
    val oceanColorArray = oceanColor.toFloatArray()
    val contourColorArray = contourColor.toFloatArray()

    companion object {
        fun default() =
            GlobeColors(
                landColor = Color(0xFF294D73),
                oceanColor = Color(0xFF192E45),
                contourColor = Color(0xFF192E45),
                backgroundColor = Color(0xFF101823),
            )
    }
}
