package net.mullvad.mullvadvpn.compose.map.data

import kotlin.math.absoluteValue
import kotlin.math.pow
import kotlin.math.sqrt
import net.mullvad.mullvadvpn.compose.map.internal.COMPLETE_ANGLE

data class LatLng(val latitude: Latitude, val longitude: Longitude) {

    fun distanceTo(other: LatLng): Double =
        sqrt(
            (latitude.distanceTo(other.latitude).absoluteValue).pow(2f) +
                ((longitude.distanceTo(other.longitude).absoluteValue).pow(2f)
        ).toDouble())

    operator fun plus(other: LatLng) =
        LatLng(latitude + other.latitude, longitude + other.longitude)

    operator fun minus(other: LatLng) =
        LatLng(latitude - other.latitude, longitude - other.longitude)
}

val gothenburgLatLng = LatLng(Latitude(57.7065f), Longitude(11.967f))

@JvmInline
value class Latitude(val value: Float) {
    init {
        require(value in LATITUDE_RANGE) {
            "Latitude must be between $MIN_LATITUDE_VALUE and $MAX_LATITUDE_VALUE"
        }
    }

    fun distanceTo(other: Latitude) = (other.value - value).absoluteValue

    operator fun plus(other: Latitude) = Latitude(value + other.value)

    operator fun minus(other: Latitude) = Latitude(value - other.value)

    companion object {
        private const val MIN_LATITUDE_VALUE: Float = -COMPLETE_ANGLE / 4 // -90
        private const val MAX_LATITUDE_VALUE: Float = COMPLETE_ANGLE / 4 // 90
        private val LATITUDE_RANGE = MIN_LATITUDE_VALUE..MAX_LATITUDE_VALUE
    }
}

@JvmInline
value class Longitude(val value: Float) {
    init {
        require(value in LONGITUDE_RANGE) {
            "Longitude must be between $MIN_LONGITUDE_VALUE and $MAX_LONGITUDE_VALUE"
        }
    }

    fun distanceTo(other: Longitude) = vectorTo(other).value.absoluteValue

    fun vectorTo(other: Longitude): Longitude {
        val diff = other.value - value
        val vectorValue =
            when {
                diff > MAX_LONGITUDE_VALUE -> diff - COMPLETE_ANGLE
                diff < MIN_LONGITUDE_VALUE -> diff + COMPLETE_ANGLE
                else -> diff
            }
        return Longitude(vectorValue)
    }

    operator fun plus(other: Longitude) = Longitude(value + other.value)

    operator fun minus(other: Longitude) = Longitude(value - other.value)

    companion object {
        private const val MIN_LONGITUDE_VALUE: Float = -COMPLETE_ANGLE / 2 // -180
        private const val MAX_LONGITUDE_VALUE: Float = COMPLETE_ANGLE / 2 // 180
        private val LONGITUDE_RANGE = MIN_LONGITUDE_VALUE..MAX_LONGITUDE_VALUE

        fun fromFloat(value: Float): Longitude {
            val unwoundValue = unwind(value)
            return Longitude(unwoundValue)
        }

        private fun unwind(value: Float): Float {
            val unwound = value % COMPLETE_ANGLE
            return when {
                unwound > MAX_LONGITUDE_VALUE -> unwound - COMPLETE_ANGLE
                unwound < MIN_LONGITUDE_VALUE -> unwound + COMPLETE_ANGLE
                else -> unwound
            }
        }
    }
}
