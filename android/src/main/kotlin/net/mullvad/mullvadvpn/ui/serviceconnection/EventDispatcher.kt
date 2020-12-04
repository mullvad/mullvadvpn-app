package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Handler
import android.os.Looper
import android.os.Message
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.withLock
import net.mullvad.mullvadvpn.service.Event

class EventDispatcher(looper: Looper) : Handler(looper) {
    private val handlers = HashMap<Event.Type, (Event) -> Unit>()
    private val lock = ReentrantReadWriteLock()

    fun <E : Event> registerHandler(eventType: Event.Type, handler: (E) -> Unit) {
        lock.writeLock().withLock {
            handlers.put(eventType) { event ->
                @Suppress("UNCHECKED_CAST")
                handler(event as E)
            }
        }
    }

    override fun handleMessage(message: Message) {
        lock.readLock().withLock {
            val event = Event.fromMessage(message)
            val handler = handlers.get(event.type)

            handler?.invoke(event)
        }
    }
}
