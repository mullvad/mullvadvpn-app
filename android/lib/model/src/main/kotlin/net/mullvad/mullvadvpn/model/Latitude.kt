package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue

@JvmInline
value class Latitude(val value: Float) {
    init {
        require(value in LATITUDE_RANGE) {
            "Latitude: '$value' must be between $MIN_LATITUDE_VALUE and $MAX_LATITUDE_VALUE"
        }
    }

    fun distanceTo(other: Latitude) = (other.value - value).absoluteValue

    operator fun plus(other: Latitude) = fromFloat(value + other.value)

    operator fun minus(other: Latitude) = fromFloat(value - other.value)

    companion object {
        private const val MIN_LATITUDE_VALUE: Float = -90f
        private const val MAX_LATITUDE_VALUE: Float = 90f
        private val LATITUDE_RANGE = MIN_LATITUDE_VALUE..MAX_LATITUDE_VALUE

        /**
         * Create a [Latitude] from a float value.
         *
         * This function will unwind a float to a valid latitude value. E.g 190 will be unwound to
         * -10 and 360 will be unwound to 0.
         */
        fun fromFloat(value: Float): Latitude {
            val unwoundValue = unwind(value)
            return Latitude(unwoundValue)
        }

        private fun unwind(value: Float): Float {
            // Remove all 360 degrees
            val withoutRotations = value % COMPLETE_ANGLE

            // If we are above 180 or below -180, we wrapped half a turn and need to flip sign
            val partiallyUnwound =
                if (withoutRotations.absoluteValue > COMPLETE_ANGLE / 2) {
                    -withoutRotations % (COMPLETE_ANGLE / 2)
                } else withoutRotations

            return when {
                partiallyUnwound < MIN_LATITUDE_VALUE ->
                    MIN_LATITUDE_VALUE - (partiallyUnwound % MIN_LATITUDE_VALUE)
                partiallyUnwound > MAX_LATITUDE_VALUE ->
                    MAX_LATITUDE_VALUE - (partiallyUnwound % MAX_LATITUDE_VALUE)
                // partiallyUnwound in range MIN_LATITUDE_VALUE..MAX_LATITUDE_VALUE
                else -> partiallyUnwound
            }
        }
    }
}
