package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
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
}

object GlobeDefaults {
    @Composable
    fun colors() =
        GlobeColors(
            landColor = MaterialTheme.colorScheme.primary,
            oceanColor = MaterialTheme.colorScheme.surface,
            backgroundColor = MaterialTheme.colorScheme.tertiary,
        )
}
