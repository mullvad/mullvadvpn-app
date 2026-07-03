package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.model.LatLong

data class Hop(
    val from: LatLong,
    val to: LatLong,
    val color: Color = Color.White.copy(alpha = DEFAULT_ALPHA),
) {
    companion object {
        const val DEFAULT_ALPHA = 0.6f
    }
}
