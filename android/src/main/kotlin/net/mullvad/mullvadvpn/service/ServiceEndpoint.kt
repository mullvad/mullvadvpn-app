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
            listeners.add(request.listener)
        }
    }

    fun onDestroy() {
        dispatcher.onDestroy()
        settingsListener.onDestroy()
    }
}
