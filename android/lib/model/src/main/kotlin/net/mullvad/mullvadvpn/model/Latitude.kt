package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue

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