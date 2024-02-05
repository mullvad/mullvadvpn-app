package net.mullvad.lib.map.data

import kotlin.math.sqrt

data class LatLng(val latitude: Float, val longitude: Float) {

    fun length() = sqrt(latitude * latitude + longitude * longitude)

    fun scale(ratio: Float) = LatLng(latitude * ratio, longitude * ratio)

    operator fun plus(other: LatLng) =
        LatLng(latitude + other.latitude, longitude + other.longitude)
}

val gothenburgLatLng = LatLng(57.7089f, 11.9746f)
