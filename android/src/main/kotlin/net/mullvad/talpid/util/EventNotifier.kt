package net.mullvad.talpid.util

class EventNotifier<T>(private val initialValue: T) {
    private val listeners = HashMap<Any, (T) -> Unit>()

    private var idCounter = 0
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

    fun subscribe(listener: (T) -> Unit): Int {
        synchronized(this) {
            val id = idCounter

            idCounter += 1
            subscribe(id, listener)

            return id
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
