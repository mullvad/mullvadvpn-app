package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.map.internal.toFloatArray

data class GlobeColors(
    val landColor: Color,
    val oceanColor: Color,
    val contourColor: Color,
) {
    val landColorArray = landColor.toFloatArray()
    val oceanColorArray = oceanColor.toFloatArray()
    val contourColorArray = contourColor.toFloatArray()
}
