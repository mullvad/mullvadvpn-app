package net.mullvad.mullvadvpn.util

// Calculates a series of delays that increase exponentially.
//
// The delays follow the formula:
//
// (base ^ retryAttempt) * scale
//
// but it is never larger than the specified cap value.
class ExponentialBackoff : Iterator<Long> {
    private var unscaledValue = 1L
    private var current = 1L

    var iteration = 1
        private set

    var base = 2L
    var scale = 1000L
    var cap = Long.MAX_VALUE
    var count: Int? = null

    override fun hasNext(): Boolean {
        val maxIterations = count

        if (maxIterations != null) {
            return iteration < maxIterations
        } else {
            return true
        }
    }

    override fun next(): Long {
        iteration += 1

        if (current >= cap) {
            return cap
        } else {
            val value = current

            unscaledValue *= base
            current = Math.min(cap, scale * unscaledValue)

            return value
        }
    }

    fun reset() {
        unscaledValue = 1L
        current = 1L
        iteration = 1
    }
}
