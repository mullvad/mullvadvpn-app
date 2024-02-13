package net.mullvad.mullvadvpn.model

import kotlin.math.absoluteValue

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

    operator fun plus(other: Longitude) = fromFloat(value + other.value)

    operator fun minus(other: Longitude) = fromFloat(value - other.value)

    companion object {
        private const val MIN_LONGITUDE_VALUE: Float = -180f
        private const val MAX_LONGITUDE_VALUE: Float = 180f
        private val LONGITUDE_RANGE = MIN_LONGITUDE_VALUE..MAX_LONGITUDE_VALUE

        /**
         * Create a [Longitude] from a float value.
         *
         * This function will unwind a float to a valid longitude value. E.g 190 will be unwound to
         * -170 and 360 will be unwound to 0.
         */
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
