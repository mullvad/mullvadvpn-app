package net.mullvad.lib.map.data

import kotlin.math.absoluteValue
import kotlin.math.pow
import kotlin.math.sqrt

data class LatLng(val latitude: Float, val longitude: Float) {

    fun distanceTo(other: LatLng): Double =
        sqrt(
            ((latitude.toDouble() - other.latitude.toDouble()).absoluteValue).pow(2.0) +
                ((longitude.toDouble() - other.longitude.toDouble()).absoluteValue).pow(2.0)
        )

    fun scale(ratio: Float) = LatLng(latitude * ratio, longitude * ratio)

    operator fun plus(other: LatLng) =
        LatLng(latitude + other.latitude, longitude + other.longitude)

    operator fun minus(other: LatLng) =
        LatLng(latitude + other.latitude, longitude + other.longitude)
}

val gothenburgLatLng = LatLng(57.7065f, 11.967f)
