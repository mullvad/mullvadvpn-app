package net.mullvad.mullvadvpn.util

import kotlin.properties.Delegates.observable
import kotlin.reflect.KProperty

class LinearInterpolation {
    private val observer = { _: KProperty<*>, oldValue: Float, newValue: Float ->
        if (!updated && oldValue != newValue) {
            updated = true
        }
    }

    private val realStart
        get() = start - reference

    private val realEnd
        get() = end - reference

    var reference by observable(0.0f, observer)
    var start by observable(0.0f, observer)
    var end by observable(0.0f, observer)

    var updated = true
        get() {
            if (field == true) {
                field = false
                return true
            } else {
                return false
            }
        }

    fun interpolate(progress: Float): Float {
        return progress * (realEnd - realStart) + realStart
    }

    fun progress(interpolation: Float): Float {
        val length = realEnd - realStart

        if (length == 0.0f) {
            return 0.0f
        }

        return (interpolation - realStart) / length
    }
}
