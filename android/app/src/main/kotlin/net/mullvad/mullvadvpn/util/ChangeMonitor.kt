package net.mullvad.mullvadvpn.util

import kotlin.properties.Delegates.observable

class ChangeMonitor {
    var changed = false
        private set

    fun <T> monitor(initialValue: T) =
        observable(initialValue) { _, oldValue, newValue ->
            if (oldValue != newValue) {
                changed = true
            }
        }

    fun reset() {
        changed = false
    }
}
