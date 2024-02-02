package net.mullvad.lib.map

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import net.mullvad.lib.map.data.Coordinate
import net.mullvad.mullvadvpn.model.TunnelState

@Composable
fun MullvadMap(
    modifier: Modifier,
    tunnelState: TunnelState,
    percent: Float,
    mode: Boolean,
    fov: Float
) {
    val animatedLat = animateFloatAsState(targetValue = tunnelState.location()?.latitude?.toFloat() ?: 0f, spring(stiffness = 50f))
    val animatedLon = animateFloatAsState(targetValue = tunnelState.location()?.longitude?.toFloat() ?: 0f, spring(stiffness = 50f))
    val disconnectedZoom = 1.35f
    val connectedZoom = 1.25f
    val zoom = animateFloatAsState(targetValue = if(tunnelState.isSecured()) connectedZoom else disconnectedZoom, spring(stiffness = 50f))

    MapGLShader(
        modifier = modifier,
        coordinate = Coordinate(animatedLat.value, animatedLon.value),
        zoom = zoom.value,
        percent = percent,
        mode,
        fov
    )
}
