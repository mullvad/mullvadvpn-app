package net.mullvad.mullvadvpn.lib.map.preview

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.map.InteractiveMap
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.Marker

val LAT_LOWER_BOUND = -40f
val LAT_UPPER_BOUND = 60f

@Preview
@Composable
private fun InteractiveMapPreview() {
    // Starting position
    val currentLocation = remember {
        // Berlin
        mutableStateOf(locations[9])
    }

    // Create some markers
    val markers = locations.map {
        Marker(
            id = it.toString(),
            latLong = it,
            colors =
                if (it == currentLocation.value) selectLocationMarkerColors
                else unselectLocationMarkerColors,
        )
    }

    Column(

    ) {
        Box(modifier = Modifier.size(200.dp).background(Color.Red).clickable(true) {
            currentLocation.value = locations[9]
        }){

        }
        InteractiveMap(
            currLocation = currentLocation,
            markers = markers,
            onMarkerClick = { currentLocation.value = it.latLong },
        )
    }

}

private val selectLocationMarkerColors =
    LocationMarkerColors(centerColor = Color(0xFF44AD4D.toInt()))

private val unselectLocationMarkerColors =
    LocationMarkerColors(
        perimeterColors = null,
        centerColor = Color(0xFF192E45.toInt()),
        ringBorderColor = Color(0xFFFFFFFF.toInt()),
    )
