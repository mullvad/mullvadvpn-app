package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.ui.graphics.Color

data class MapConfig(
    val globeColors: GlobeColors =
        GlobeColors(
            landColor = Color(0.16f, 0.302f, 0.45f),
            oceanColor = Color(0.098f, 0.18f, 0.271f),
            contourColor = Color(0.098f, 0.18f, 0.271f)
        ),
    val secureMarkerColor: LocationMarkerColors =
        LocationMarkerColors(centerColor = Color(0.267f, 0.678f, 0.302f)),
    val unsecureMarkerColor: LocationMarkerColors =
        LocationMarkerColors(centerColor = Color(0.89f, 0.251f, 0.224f))
)