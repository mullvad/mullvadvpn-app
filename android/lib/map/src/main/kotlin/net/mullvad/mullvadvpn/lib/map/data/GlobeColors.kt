package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.map.internal.toFloatArray

@Immutable
data class GlobeColors(
    val landColor: Color,
    val oceanColor: Color,
    val contourColor: Color = oceanColor,
) {
    val landColorArray = landColor.toFloatArray()
    val oceanColorArray = oceanColor.toFloatArray()
    val contourColorArray = contourColor.toFloatArray()
}
