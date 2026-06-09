package net.mullvad.mullvadvpn.lib.map.preview

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.map.Map
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

@Preview(showBackground = false, showSystemUi = false)
@Composable
private fun MapPreview() {
    val berlin = LatLong(Latitude(52.5200f), Longitude(13.4050f))
    Map(cameraPosition = CameraPosition(latLong = berlin, zoom = 1.9f))
}

@Preview
@Composable
private fun SpinningGlobePreview() {
    val infinite = rememberInfiniteTransition()
    val rawLongitude by
        infinite.animateFloat(
            initialValue = 0f,
            targetValue = 360f,
            animationSpec =
                infiniteRepeatable(animation = tween(durationMillis = 30000, easing = LinearEasing)),
        )
    val longitude = Longitude.fromFloat(rawLongitude)

    Map(
        cameraPosition =
            CameraPosition(
                latLong = LatLong(Latitude(0f), longitude),
                zoom = 1.9f,
                verticalBias = 0.5f,
            )
    )
}
