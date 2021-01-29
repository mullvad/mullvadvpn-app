package net.mullvad.mullvadvpn.service

import android.os.Looper
import android.os.Messenger
import net.mullvad.mullvadvpn.util.DispatchingHandler
import net.mullvad.mullvadvpn.util.Intermittent

class ServiceEndpoint(looper: Looper, intermittentDaemon: Intermittent<MullvadDaemon>) {
    private val listeners = mutableListOf<Messenger>()

    internal val dispatcher = DispatchingHandler(looper) { message ->
        Request.fromMessage(message)
    }

    val messenger = Messenger(dispatcher)

    val settingsListener = SettingsListener(intermittentDaemon)

    init {
        dispatcher.registerHandler(Request.RegisterListener::class) { request ->
            registerListener(request.listener)
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
        settingsListener.onDestroy()
    }

    private fun registerListener(listener: Messenger) {
        listeners.add(listener)

        listener.apply {
            send(Event.ListenerReady().message)
        }
    }
}
