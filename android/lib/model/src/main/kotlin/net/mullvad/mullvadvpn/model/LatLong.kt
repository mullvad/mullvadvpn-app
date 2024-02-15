package net.mullvad.mullvadvpn.model

import kotlin.math.pow
import kotlin.math.sqrt

data class LatLong(val latitude: Latitude, val longitude: Longitude) {

    fun distanceTo(other: LatLong): Float =
        sqrt(
            latitude.distanceTo(other.latitude).pow(2f) +
                (longitude.distanceTo(other.longitude).pow(2f))
        )

    operator fun plus(other: LatLong) =
        LatLong(latitude + other.latitude, longitude + other.longitude)

    operator fun minus(other: LatLong) =
        LatLong(latitude - other.latitude, longitude - other.longitude)
}

const val COMPLETE_ANGLE = 360f
