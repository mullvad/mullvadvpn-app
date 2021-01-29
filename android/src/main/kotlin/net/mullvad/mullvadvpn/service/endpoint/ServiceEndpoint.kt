package net.mullvad.mullvadvpn.service.endpoint

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
