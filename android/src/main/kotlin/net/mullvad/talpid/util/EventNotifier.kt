package net.mullvad.talpid.util

// Manages listeners interested in receiving events of type T
//
// The listeners subscribe using an ID object. This ID is used later on for unsubscribing. The only
// requirement is that the object uses the default implementation of the `hashCode` and `equals`
// methods inherited from `Any` (or `Object` in Java).
//
// If the ID object class (or any of its super-classes) overrides `hashCode` or `equals`,
// unsubscribe might not work correctly.
class EventNotifier<T>(private val initialValue: T) {
    private val listeners = HashMap<Any, (T) -> Unit>()

    private var latestEvent = initialValue

    fun notify(event: T) {
        synchronized(this) {
            for (listener in listeners.values) {
                listener(event)
            }

            latestEvent = event
        }
    }

    fun subscribe(id: Any, listener: (T) -> Unit) {
        synchronized(this) {
            listeners.put(id, listener)
            listener(latestEvent)
        }
    }

    fun hasListeners(): Boolean {
        synchronized(this) {
            return !listeners.isEmpty()
        }
    }

    fun unsubscribe(id: Any) {
        synchronized(this) {
            listeners.remove(id)
        }
    }

    fun unsubscribeAll() {
        synchronized(this) {
            listeners.clear()
        }
    }
}
