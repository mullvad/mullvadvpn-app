package net.mullvad.mullvadvpn.util

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.delay

// Helper to filter out bursts of events so that only the latest event in an interval is notified.
//
// An interval of zero means that it will only debounce events that are sent before the job is
// started. If the events are coming from the UI thread, this means that this class will only send
// the last event received before the UI thread finishes its current task.
//
// This can be used for example to filter out focus events coming from different views. Android will
// first send a "focus lost" event from a view followed by a "focus gained" event from another view.
// If the only thing the listener is interested in is if any of a set of views has focus, this class
// can be used to debounce focus events from the set of views to obtain an event that represents a
// change from when the set contains a focused view to when the set contains no focused views (and
// an event for the reverse situation).
class Debouncer<T>(initialValue: T, val intervalInMs: Long = 0) {
    private val jobTracker = JobTracker()

    var listener: ((T) -> Unit)? = null

    var debouncedValue = initialValue
        private set

    var rawValue by
        observable(initialValue) { _, oldValue, newValue ->
            if (newValue != oldValue) {
                jobTracker.cancelJob("notifyNewValue")

                if (newValue != debouncedValue) {
                    jobTracker.newUiJob("notifyNewValue") {
                        delay(intervalInMs)
                        listener?.invoke(newValue)
                        debouncedValue = newValue
                    }
                }
            }
        }
}
