package net.mullvad.mullvadvpn.lib.map.preview

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.map.InteractiveMap
import net.mullvad.mullvadvpn.lib.map.data.Hop
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.Marker

@Preview
@Composable
private fun PreviewInteractiveMapPreview() {
    // Starting position
    var currentLocation by remember {
        // Berlin
        mutableStateOf(locations[9])
    }

    // Create some markers
    val markers = locations.map {
        Marker(
            latLong = it,
            colors =
                if (it == currentLocation) selectLocationMarkerColors
                else unselectLocationMarkerColors,
        )
    }
    InteractiveMap(
        currentLocation = currentLocation,
        markers = markers,
        hops = listOf(Hop(currentLocation, locations[1]), Hop(locations[1], locations[2])),
        locations = emptyList(),
        onMarkerClick = { currentLocation = it.latLong },
    )
}

private val selectLocationMarkerColors =
    LocationMarkerColors(centerColor = Color(0xFF44AD4D.toInt()))

private val unselectLocationMarkerColors =
    LocationMarkerColors(
        perimeterColors = null,
        centerColor = Color(0xFF192E45.toInt()),
        ringBorderColor = Color(0xFFFFFFFF.toInt()),
    )
