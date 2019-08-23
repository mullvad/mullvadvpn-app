package net.mullvad.mullvadvpn.util

class EventNotifier<T>(private val initialValue: T) {
    private val listeners = HashMap<Int, (T) -> Unit>()

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

    fun subscribe(listener: (T) -> Unit): Int {
        synchronized(this) {
            val id = idCounter

            idCounter += 1
            listeners.put(id, listener)
            listener(latestEvent)

            return id
        }
    }

    fun unsubscribe(id: Int) {
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
