package net.mullvad.lib.map

import kotlin.math.sqrt

data class Coordinate(val lat: Float, val lon: Float) {

    fun length() = sqrt(lat * lat + lon * lon)

    fun scale(ratio: Float) = Coordinate(lat * ratio, lon * ratio)

    operator fun plus(other: Coordinate) = Coordinate(lat + other.lat, lon + other.lon)
}
