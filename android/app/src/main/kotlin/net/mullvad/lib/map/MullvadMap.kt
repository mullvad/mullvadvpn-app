package net.mullvad.lib.map

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import net.mullvad.lib.map.data.Coordinate

@Composable
fun MullvadMap(modifier: Modifier, coordinate: Coordinate, zoom: Float, percent: Float) {
    MapGLShader(modifier = modifier, coordinate = coordinate, zoom = zoom, percent = percent)
}
