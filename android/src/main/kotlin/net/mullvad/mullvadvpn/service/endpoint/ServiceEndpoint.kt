package net.mullvad.mullvadvpn.service.endpoint

import android.os.DeadObjectException
import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request

class ServiceEndpoint(looper: Looper) {
    private val listeners = mutableSetOf<Messenger>()

    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    init {
        dispatcher.registerHandler(Request.RegisterListener::class) { request ->
            registerListener(request.listener)
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
    }

    internal fun sendEvent(event: Event) {
        synchronized(this) {
            val deadListeners = mutableSetOf<Messenger>()

            for (listener in listeners) {
                try {
                    listener.send(event.message)
                } catch (_: DeadObjectException) {
                    deadListeners.add(listener)
                }
            }

            deadListeners.forEach { listeners.remove(it) }
        }
    }

    private fun registerListener(listener: Messenger) {
        synchronized(this) {
            listeners.add(listener)

            val initialEvents = listOf(Event.ListenerReady)

            initialEvents.forEach { event ->
                listener.send(event.message)
            }
        }
    }
}
